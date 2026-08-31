# Contributing to lily-contracts

Thanks for contributing to Lily Protocol’s Soroban contracts.

## Principles

- Keep changes small, reviewable, and tied to a single protocol concern.
- Prefer explicit state machines, typed errors, and auth checks over convenience shortcuts.
- Add or update tests for every behavior change.
- Document storage, event, and authorization implications in pull requests.

## Local setup

1. Install Rust. The repository pins the toolchain in `rust-toolchain.toml` (currently Rust 1.83), so `rustup` will automatically install the correct version when you run any cargo command in the workspace.
2. Install `stellar-cli` using the official Stellar instructions.
3. The Wasm target `wasm32v1-none` is declared in `rust-toolchain.toml`, so it will be installed automatically. If you prefer to install it manually, run `rustup target add wasm32v1-none`.
4. Run `make fmt`, `make lint`, and `make test` before opening a PR.

## Repository conventions

- `contracts/` contains deployable Soroban contracts.
- `crates/lily-common` contains shared no-std primitives used by contracts.
- `crates/lily-test-support` contains reusable test helpers only.
- Contract state keys should stay typed and local to each contract crate.
- Initialization must be one-time and explicitly tested.
- Admin actions must always require direct auth.

## Testing expectations

Every contract change should consider:

- Happy path behavior
- Unauthorized access attempts
- Initialization safety
- State transition failures
- Storage read/write expectations

## Pull requests

Please include:

- A clear problem statement
- A short summary of behavior changes
- Notes on storage layout or auth changes
- Test coverage summary
- Follow-up work if the change intentionally leaves gaps

## Security reporting

Do not open public issues for exploitable vulnerabilities. Until a dedicated security channel is published, contact the Lily Protocol maintainers privately and include reproduction steps, impact, and affected contracts.

## Good first contributions

Areas intentionally left open for contributors include:

- Additional negative-path tests
- Richer event schemas
- Contract deployment tooling
- Cross-contract integration tests
- Governance and role separation enhancements
