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

### `contracts/payments`

Payment intent and settlement primitive. Tracks payment intents, allows payer-side cancellation, and supports admin-driven settlement finalization.

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
```

## Contract development approach

This repository intentionally ships a real, reviewable foundation without prematurely implementing every protocol feature. The current contracts establish:

- Typed storage keys and typed return structs
- Explicit initialization guards
- Auth-gated admin and actor actions
- Event emission on state transitions
- Conservative state transitions for settlement lifecycle
- Clear separation between protocol domains

## Compatibility policies

- [Event compatibility policy](./docs/EVENT_COMPATIBILITY.md) — additive and versioned change rules for topics and payloads consumed by indexers

## Future protocol areas intentionally left for follow-up

- On-chain agent reputation and credential attestations
- Multi-role governance and timelocked admin changes
- Escrowed settlement and token transfer integration
- Cross-contract composition between identity, wallet, and payments
- Upgrade and migration playbooks
- Fuzzing, invariants, and deeper adversarial testing
- Mainnet deployment manifests and release signing

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) before opening a pull request. Contributors should keep changes scoped to a clear protocol concern and include tests for any state transition, auth path, or storage behavior they modify.

## Security

This is smart contract infrastructure. Avoid introducing:

- Implicit authorization paths
- Unbounded storage growth without design review
- Silent state overwrites
- Incomplete initialization or upgrade assumptions
- Panic-driven business logic where typed errors are more appropriate

If you believe you’ve found a vulnerability, please follow the security guidance in [CONTRIBUTING.md](./CONTRIBUTING.md).
