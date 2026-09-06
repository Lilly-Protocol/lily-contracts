# lily-contracts

[![CI](https://github.com/lily-protocol/lily-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/lily-protocol/lily-contracts/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](./LICENSE)
[![Soroban](https://img.shields.io/badge/Stellar-Soroban-green)](https://developers.stellar.org/docs/build/smart-contracts)

Production-oriented Soroban smart contracts for Lily Protocol on Stellar.

`lily-contracts` is the on-chain smart contract repository for Lily Protocol, an autonomous agent finance stack on Stellar. This workspace provides the protocol foundation for agent identity, wallet policy, payment settlement primitives, and global protocol configuration, with contributor-friendly structure for future protocol expansion.

## Why this repository exists

Lily Protocol needs contract infrastructure that is secure, modular, and understandable by external contributors. This repository is designed to support:

- Agent registration and identity records
- Wallet and policy binding for AgentLily-controlled accounts
- Payment intent creation and asynchronous settlement flows
- Protocol-wide configuration and admin controls
- Shared storage, event, and error conventions across contracts

## Workspace layout

```text
.
├── contracts
│   ├── identity
│   ├── payments
│   ├── protocol
│   └── wallet
├── crates
│   ├── lily-common
│   └── lily-test-support
├── .github
│   ├── ISSUE_TEMPLATE
│   └── workflows
├── scripts
├── Cargo.toml
└── Makefile
```

## Contracts and crates

### `contracts/protocol`

Global protocol configuration contract. Handles one-time initialization, admin transfer, fee basis points, and treasury configuration.

### `contracts/identity`

Agent identity registry. Supports protocol bootstrapping, agent registration, controller rotation, metadata updates, and admin deactivation.

### `contracts/wallet`

Wallet policy registry. Maintains agent-to-wallet bindings, settlement asset symbols, spend limits, and enabled state toggles.

Binding lifecycle:
- `bind_wallet` creates a brand-new binding and fails if the agent already has one.
- `rebind_wallet` explicitly replaces an existing binding (enabled or disabled) with a new wallet, asset, and spend limit, resetting revision to 0.
- `update_spend_limit` and `set_enabled` mutate the current binding without replacing it.

This split prevents the silent state overwrites that would occur if `bind_wallet` were reused after a binding had been disabled.

### `contracts/payments`

Payment intent and settlement primitive. Tracks payment intents, allows payer-side cancellation, and supports admin-driven settlement finalization.

See [Payment intent indexes](./docs/PAYMENT_INDEXES.md) for payer pagination semantics and storage-cost considerations.

### `crates/lily-common`

Shared contract utilities, typed protocol errors, payment status enum, basis point guards, and storage TTL helpers.

### `crates/lily-test-support`

Reusable Soroban test helpers for local environments, synthetic addresses, and string conversion.

## Documentation

- [Authorization model](docs/AUTH.md) — function-by-function authorization matrix for every public contract function, with the reasoning behind each choice.

## Local requirements

- Rust toolchain with `cargo` and `rustfmt`
- `clippy` component available for linting
- `stellar-cli` for contract artifact workflows and deployment
- `wasm32v1-none` target installed for Wasm builds

Official Stellar docs currently recommend:

- A Rust workspace with contracts under `contracts/*`
- `soroban-sdk = "22"` for current Soroban contracts
- `stellar-cli` installation via `brew install stellar-cli`, `cargo install --locked stellar-cli`, or the Stellar installer

## Getting started

```bash
git clone https://github.com/lily-protocol/lily-contracts.git
cd lily-contracts
make fmt
make test
```

If you need the CLI locally:

```bash
brew install stellar-cli
```

If you need the Wasm target:

```bash
rustup target add wasm32v1-none
```

You can inspect the local toolchain status with:

```bash
./scripts/check-tooling.sh
```

Before deploying or invoking contracts, check that a Soroban RPC endpoint is responding:

```bash
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org ./scripts/rpc-health.sh
```

The health probe calls both `getHealth` and `getLatestLedger` and exits non-zero on an HTTP, transport, or JSON-RPC failure.

## Common development commands

```bash
make fmt
make fmt-check
make lint
make check
make test
make build
make build-wasm
make artifacts
make ci

The `make artifacts` target also generates `dist/manifest.json` with sha256 hashes, package versions, git commit, and build profile for each Wasm artifact.
```

The lint target runs clippy with `--all-features` and the pedantic group enabled. A workspace allow-list suppresses stylistic lints that are not actionable for this codebase (`must_use_candidate`, `needless_pass_by_value`, `similar_names`, `missing_panics_doc`, `should_panic_without_expect`).

## Contract development approach

This repository intentionally ships a real, reviewable foundation without prematurely implementing every protocol feature. The current contracts establish:

- Typed storage keys and typed return structs
- Explicit initialization guards
- Auth-gated admin and actor actions
- Event emission on state transitions (see [docs/EVENTS.md](./docs/EVENTS.md))
- Fee configuration in basis points with a documented treasury role (see [docs/FEES.md](./docs/FEES.md))
- Conservative state transitions for settlement lifecycle
- Clear separation between protocol domains
- Documented storage layouts, TTL policy, and auth model (see [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md))

## Compatibility policies

- [Event compatibility policy](./docs/EVENT_COMPATIBILITY.md) — additive and versioned change rules for topics and payloads consumed by indexers

See [Contract Testing](./docs/TESTING.md) for guidance on mock authorization,
real-auth negative tests, and the current authorization coverage debt.

For the signer requirements of every contract entry point and the reasoning
behind each role boundary, see the [authorization model](./docs/AUTH.md).

## Future protocol areas intentionally left for follow-up

- On-chain agent reputation and credential attestations
- Multi-role governance and timelocked admin changes
- Escrowed settlement and token transfer integration
- Cross-contract composition between identity, wallet, and payments
- Upgrade and migration playbooks
- Fuzzing, invariants, and deeper adversarial testing
- Mainnet deployment manifests and release signing

## Documentation

- [CONTRIBUTING.md](./CONTRIBUTING.md) — setup, conventions, and PR expectations.
- [docs/ERRORS.md](./docs/ERRORS.md) — protocol error codes, raise sites, and triggering conditions.

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) before opening a pull request and
review the project's [CHANGELOG.md](./CHANGELOG.md) for unreleased contract and
tooling changes. Contributors should keep changes scoped to a clear protocol
concern and include tests for any state transition, auth path, or storage
behavior they modify.

## Security

This is smart contract infrastructure. Avoid introducing:

- Implicit authorization paths
- Unbounded storage growth without design review
- Silent state overwrites
- Incomplete initialization or upgrade assumptions
- Panic-driven business logic where typed errors are more appropriate

If you believe you’ve found a vulnerability, please follow the security guidance in [CONTRIBUTING.md](./CONTRIBUTING.md).
