# Contributing to lily-contracts

Thanks for contributing to Lily Protocol’s Soroban contracts.

## Principles

- Keep changes small, reviewable, and tied to a single protocol concern.
- Prefer explicit state machines, typed errors, and auth checks over convenience shortcuts.
- Add or update tests for every behavior change.
- Document storage, event, and authorization implications in pull requests.

## Local setup

1. Install Rust and verify `cargo --version`.
2. Install `stellar-cli` using the official Stellar instructions.
   Use a release from the same major line as the workspace
   `soroban-sdk` (currently the `v22` line; CI pins
   `stellar/stellar-cli@v22.8.2`). `scripts/check-tooling.sh`
   fails CI when the CLI and SDK major versions drift apart.
3. Install the Wasm target with `rustup target add wasm32v1-none`.
4. Run `make fmt`, `make lint`, and `make test` before opening a PR.

### Deterministic builds and Cargo.lock

All cargo targets in the Makefile and CI workflows execute with the `--locked` flag to ensure deterministic dependency resolution and detect lockfile drift. If you add or update dependencies in `Cargo.toml`, update `Cargo.lock` explicitly (e.g. via `cargo check`) and commit both files together. If `Cargo.lock` is out of sync with `Cargo.toml`, `--locked` builds will fail with `error: the lock file ... needs to be updated but --locked was passed to prevent this`.

## Repository conventions

- `contracts/` contains deployable Soroban contracts.
- `crates/lily-common` contains shared no-std primitives used by contracts.
- `crates/lily-test-support` contains reusable test helpers only.
- Contract state keys should stay typed and local to each contract crate.
- Initialization must be one-time and explicitly tested.
- Admin actions must always require direct auth.
- Perform authorization before validating caller-supplied input in initialization and
  admin-gated functions. This prevents unauthenticated callers from probing validation
  outcomes; state-existence checks needed to resolve the stored admin may run first.

## Initialization and deployer trust

Every contract pins its intended initial admin in the `__constructor`, which
the deployer supplies at deploy time:

```rust
let contract_id = env.register(ProtocolContract, (initial_admin,));
```

`initialize` only accepts an `admin` argument that matches this pinned
address; any other caller fails with `ProtocolError::Unauthorized`. Because
the pin is written before the contract address is publicly known, a
front-runner cannot claim a fresh deployment by calling `initialize` first
with their own address. The trust model is therefore: the identity of the
initial admin is fixed at deploy time, and the first `initialize` call must
use exactly that address.

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

## Event compatibility review gate

Event topics and payloads are public interfaces. Any pull request that changes an event must follow [the event compatibility policy](./docs/EVENT_COMPATIBILITY.md), list the affected schemas, preserve existing topics, and include exact topic and payload assertions. Breaking event changes require a versioned event and a documented migration path.

## Security reporting

Do not open public issues for exploitable vulnerabilities. Until a dedicated security channel is published, contact the Lily Protocol maintainers privately and include reproduction steps, impact, and affected contracts.

## Continuous Integration and Scheduled Builds

The repository executes automated CI on all pull requests and pushes to `main`/`master`. In addition, a scheduled nightly workflow runs at `02:00 UTC` against the latest toolchain to detect upstream toolchain drifts or compiler regressions early.

If a scheduled nightly run fails:
- The maintainers review the failure logs to identify whether an upstream dependency or toolchain update introduced a breaking change.
- A tracking issue is opened to pin or adapt to the toolchain revision before it affects developer pull requests.

## Good first contributions

Areas intentionally left open for contributors include:

- Additional negative-path tests
- Richer event schemas
- Contract deployment tooling
- Cross-contract integration tests
- Governance and role separation enhancements

## Auth and error mapping

Soroban enforces cryptographic signatures in the host, not in contract code.
Consequently an authorization failure can surface in **two different shapes**,
and integrators should treat both as "not allowed":

| Failure class | What the SDK raises | How this repo produces it |
| --- | --- | --- |
| Missing / invalid **signature** | Host `Auth` error (`unwrap_infallible` trap from `Address::require_auth`) | `lily_common::require_auth_or_error(&addr, &env)` — the single canonical signature-check entry point used by every contract |
| Wrong **role** (caller is not the expected principal) | Typed `ProtocolError::Unauthorized` (`Error(Contract, #3)`) | `lily_common::require_caller(&env, &caller, &expected)` — call this *before* `require_auth_or_error` whenever the contract knows the expected principal |
| Reentrant invocation into a guarded transition | Typed `ProtocolError::ReentrantCall` (`Error(Contract, #10)`) | `lily_common::NonReentrantGuard::acquire(&env, key)` — see `SECURITY.md` |

Notes for off-chain consumers:

- `ProtocolError` discriminants are stable wire identifiers; match on them, not on panic strings.
- A host `Auth` error at the top of the call stack means the presented authorizer did not sign the call — map it to `Unauthorized` in application code.
- Prefer typed role checks for every "who is allowed" question: they produce
  structured `ContractError`s that survive the contract boundary, whereas the
  `Auth` trap is indistinguishable across different authorization rules.
- Example: `payments::settle_intent` first runs `require_caller` (typed
  `Unauthorized` for a non-admin caller) and only then `require_auth_or_error`
  (host `Auth` error for a non-signing admin).

### Reentrancy guard usage

State-transition functions (settle, cancel, any future escrow release) hold a
`NonReentrantGuard` across their mutation window:

```rust
let _guard = NonReentrantGuard::acquire(&env, symbol_short!("settle"));
// ...transition logic...
```

Rules:

- Use one guard **key per transition** (`Symbol`, unique within the contract's
  instance storage) so guarded windows never collide with business keys.
- The guard is released on scope exit **including panic unwind**, so the flag
  never leaks across calls.
- The Soroban 22 host already rejects direct re-invocation of a contract that
  is on the call stack; the guard is the shared, typed, cross-SDK
  defense-in-depth layer for recursive acquisition and for SDK builds that
  allow reentry.
