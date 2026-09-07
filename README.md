# Lily Contracts — Soroban Smart Contracts on Stellar

[![CI](https://github.com/lily-protocol/lily-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/lily-protocol/lily-contracts/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](./LICENSE)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-7D00FF)](https://developers.stellar.org/docs/build/smart-contracts)
[![Rust](https://img.shields.io/badge/Rust-stable-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)

**Soroban smart contracts for Lily Protocol — the Stellar-native finance layer that gives autonomous AI agents on-chain identity, wallet policy, and payment settlement.**

`lily-contracts` is the on-chain home of Lily Protocol on the **Stellar network**. It is a Rust/Soroban workspace that turns an autonomous agent (an "AgentLily") into a first-class Stellar participant: the agent is **registered on-chain**, bound to a **Stellar wallet** with policy and spend limits, and settles **payment intents** against Stellar assets — all governed by typed storage, explicit authorization, and emitted events that Stellar indexers can consume.

The contracts are intentionally written as a secure, modular, contributor-reviewable foundation rather than a closed monolith — every state transition, auth path, and fee rule is documented and tested so external contributors can extend the protocol safely.

> **Stellar ecosystem building block.** This workspace follows the Soroban development model recommended by Stellar — contracts under `contracts/*`, `soroban-sdk` at workspace level, `stellar-cli` for builds/deploys, and the `wasm32v1-none` target for Wasm artifacts that deploy to Stellar testnet and mainnet.

---

## Deep Stellar Integration

Lily Protocol's on-chain surface maps cleanly onto the Stellar ledger:

- **On-chain agent identity** — every agent registers a Stellar `Address` (its AgentLily controller account) in `contracts/identity`, with controller rotation and metadata updates recorded on-chain.
- **Stellar wallet policy** — `contracts/wallet` binds each agent to a **settlement asset symbol** and a spend limit, so an agent can only transact within the policy its controller authorized.
- **Payment intents & settlement** — `contracts/payments` lets agents create intents that are **cancelled by the payer** and **finalized by admin settlement**, with fee basis points and a treasury role configured for the Stellar asset flow.
- **SEP/asset-ready primitives** — settlement asset symbols, typed error codes, and storage-TTL helpers live in `crates/lily-common`, ready for USDC (Stellar Asset Contract) and native XLM integration.
- **Escrow & milestones (in design)** — `contracts/escrow/` carries invariant test waves for escrowed settlement: milestone payout ratios, dispute windows, multisig quorum, timelock pause, and royalty distribution.

```
┌────────────────────────────────────────────────────────────────────┐
│  AgentLily (autonomous finance agent on Stellar)                    │
│    identity  ── registered on-chain (contracts/identity)           │
│    wallet    ── bound to Stellar address + policy (contracts/wallet)│
│    payments  ── creates intents, awaits settlement (contracts/payments)│
└───────────────────────────────┬────────────────────────────────────┘
                                │ Soroban auth (require_auth on Stellar Address)
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│  contracts/  (Rust · Soroban SDK, workspace-managed)               │
│   protocol    global config: admin, treasury, fee basis points     │
│   identity    agent registry: register, controller rotation        │
│   wallet      policy: bind_wallet, spend limits, enable/disable    │
│   payments    intents: create / settle / cancel, payer + admin     │
│   escrow      design waves: milestones, disputes, quorum, royalty  │
│  crates/                                                           │
│   lily-common        typed errors, status enums, TTL helpers       │
│   lily-test-support  Soroban test utilities for local dev          │
└───────────────────────────────┬────────────────────────────────────┘
                                │ compiled to Wasm, deployed with stellar-cli
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│  Stellar network (testnet / mainnet)                               │
│  Soroban RPC · Stellar Asset Contracts · ledger events & storage   │
└────────────────────────────────────────────────────────────────────┘
```

---

## Workspace layout

```text
.
├── contracts
│   ├── identity        # Agent identity registry
│   ├── payments        # Payment intent & settlement primitive
│   ├── protocol        # Global protocol configuration
│   ├── wallet          # Wallet/policy binding registry
│   └── escrow          # Escrow design waves (invariant test specs)
├── crates
│   ├── lily-common         # Shared contract utilities & errors
│   └── lily-test-support   # Reusable Soroban test helpers
├── .github
│   ├── ISSUE_TEMPLATE
│   └── workflows
├── scripts
├── Cargo.toml
└── Makefile
```

## Contracts and crates

### `contracts/protocol`

Global protocol configuration contract. Handles one-time initialization, admin transfer, fee basis points, and treasury configuration — the shared governance root of the on-chain protocol.

### `contracts/identity`

Agent identity registry on Stellar. Supports protocol bootstrapping, **agent registration keyed by Stellar `Address`**, controller rotation, metadata updates, and admin deactivation.

### `contracts/wallet`

Wallet policy registry. Maintains **agent-to-Stellar-wallet bindings**, settlement asset symbols, spend limits, and enabled-state toggles.

Binding lifecycle:

- `bind_wallet` creates a brand-new binding and fails if the agent already has one.
- `rebind_wallet` explicitly replaces an existing binding (enabled or disabled) with a new wallet, asset, and spend limit, resetting revision to 0.
- `update_spend_limit` and `set_enabled` mutate the current binding without replacing it.

This split prevents the silent state overwrites that would occur if `bind_wallet` were reused after a binding had been disabled.

### `contracts/payments`

Payment intent and settlement primitive. Tracks payment intents, allows payer-side cancellation, and supports admin-driven settlement finalization with a fee-aware treasury.

See [Payment intent indexes](./docs/PAYMENT_INDEXES.md) for payer pagination semantics and storage-cost considerations.

### `contracts/escrow`

Escrow settlement design workstream. This contract is intentionally not yet compiled into the workspace: it currently ships as **invariant test waves** (`wave3` escrow invariants, `wave4` multisig quorum, `wave5` tiered milestones, `wave6` timelock/pause, `wave7` royalty distribution) that pin the financial math before the entry points are finalized.

### `crates/lily-common`

Shared contract utilities, typed protocol errors, payment status enum, basis point guards, and storage TTL helpers.

### `crates/lily-test-support`

Reusable Soroban test helpers for local environments, synthetic addresses, and string conversion.

## Documentation

- [Authorization model](docs/AUTH.md) — function-by-function authorization matrix for every public contract function, with the reasoning behind each choice.
- [Events](docs/EVENTS.md) — event emission on state transitions.
- [Fees](docs/FEES.md) — fee configuration in basis points with a documented treasury role.
- [Storage architecture](docs/ARCHITECTURE.md) — storage layouts, TTL policy, and auth model.
- [Event compatibility policy](docs/EVENT_COMPATIBILITY.md) — additive and versioned change rules for topics and payloads consumed by indexers.
- [Contract testing](docs/TESTING.md) — mock-authorization, real-auth negative tests, and current authorization coverage debt.
- [Protocol errors](docs/ERRORS.md) — protocol error codes, raise sites, and triggering conditions.

## Local requirements

- Rust toolchain with `cargo` and `rustfmt`
- `clippy` component available for linting
- `stellar-cli` for contract artifact workflows and deployment
- `wasm32v1-none` target installed for Wasm builds

Official Stellar docs currently recommend:

- A Rust workspace with contracts under `contracts/*`
- `soroban-sdk` at workspace level for current Soroban contracts
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
make verify
make test-manifest
npm run test
make ci
```

The `make artifacts` target also generates `dist/manifest.json` with sha256 hashes, package versions, git commit, and build profile for each Wasm artifact.

The lint target runs clippy with `--all-features` and the pedantic group enabled. A workspace allow-list suppresses stylistic lints that are not actionable for this codebase (`must_use_candidate`, `needless_pass_by_value`, `similar_names`, `missing_panics_doc`, `should_panic_without_expect`).

## Contract development approach

This repository intentionally ships a real, reviewable foundation without prematurely implementing every protocol feature. The current contracts establish:

- Typed storage keys and typed return structs
- Explicit initialization guards
- Auth-gated admin and actor actions (`require_auth` on Stellar `Address`es)
- Event emission on state transitions
- Fee configuration in basis points with a documented treasury role
- Conservative state transitions for settlement lifecycle
- Clear separation between protocol domains
- Documented storage layouts, TTL policy, and auth model

## Future protocol areas intentionally left for follow-up

- **Escrow contract entry points** beyond the invariant test waves
- On-chain agent reputation and credential attestations
- Multi-role governance and timelocked admin changes
- **Token-transfer settlement** with USDC / native XLM via Stellar Asset Contracts
- Cross-contract composition between identity, wallet, and payments
- Upgrade and migration playbooks
- Fuzzing, invariants, and deeper adversarial testing
- Mainnet deployment manifests and release signing

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) before opening a pull request and review the project's [CHANGELOG.md](./CHANGELOG.md) for unreleased contract and tooling changes. Contributors should keep changes scoped to a clear protocol concern and include tests for any state transition, auth path, or storage behavior they modify.

## Security

This is smart contract infrastructure on Stellar. Avoid introducing:

- Implicit authorization paths
- Unbounded storage growth without design review
- Silent state overwrites
- Incomplete initialization or upgrade assumptions
- Panic-driven business logic where typed errors are more appropriate

If you believe you've found a vulnerability, please follow the security guidance in [CONTRIBUTING.md](./CONTRIBUTING.md).
