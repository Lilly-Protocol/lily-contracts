# Testing contracts

Lily contract tests use helpers from `crates/lily-test-support`. The shared `test_env()` helper currently calls `env.mock_all_auths()`, which is convenient for happy-path and state-machine tests but must not be treated as proof that authorization rules are correct.

## Why `mock_all_auths()` needs care

`mock_all_auths()` makes every `Address::require_auth()` and `Address::require_auth_for_args()` invocation succeed. That keeps unrelated authorization setup out of ordinary behavior tests, but it also means an authorization regression can be hidden by the test environment.

For example, a test created with `test_env()` can still pass when it exercises an admin-only or payer-only path, because the environment supplies authorization automatically. A happy-path assertion therefore verifies the state transition, not the caller boundary.

Use blanket auth mocking when authorization is not the behavior under test, such as:

- storage and state-transition behavior after a valid call;
- input validation that is independent of caller identity;
- event payloads and read-only queries;
- multi-step happy paths where auth setup would obscure the behavior being tested.

Do not use blanket auth mocking as the only coverage for:

- initialization guarded by an administrator;
- admin-only configuration or settlement actions;
- payer, owner, or account-specific mutations;
- any change that adds, removes, or moves a `require_auth()` call.

## Writing authorization tests

Authorization-focused tests should start from `Env::default()` instead of `lily_test_support::test_env()`.

Use narrowly scoped authorization for any setup call that genuinely needs it. Soroban's `mock_auths()` can authorize the exact address and invocation required for setup; authorizations that are not listed do not pass. Then invoke the protected action without the required authorization and assert that the call fails.

A negative-auth test should make the boundary obvious:

1. Create a fresh `Env::default()`.
2. Register the contract and construct its client.
3. Authorize only the setup invocations required to reach the state being tested.
4. Do **not** authorize the address required by the protected action.
5. Call the protected action and assert an authorization failure (using the generated `try_*` client method when practical, or an expected panic when that better matches the existing test style).
6. If state could have changed before the auth check, also assert that the failed call left storage unchanged.

For positive authorization coverage, prefer `mock_auths()` with the expected address/invocation over `mock_all_auths()`. When a broader mock is unavoidable, inspect `env.auths()` after the call and assert that the expected authorization tree was actually requested. This prevents a missing `require_auth()` from silently turning a mocked test green.

## Current test-suite debt

The current shared `test_env()` enables `mock_all_auths()` for every caller, so the existing suite is primarily behavior coverage rather than complete authorization coverage. This is known test debt, not an authorization guarantee.

The migration plan is incremental:

- new or changed auth-sensitive entrypoints should include a negative-auth test;
- prioritize admin, payment, wallet, ownership, and other value-moving mutations;
- replace blanket mocks with `mock_auths()` in auth-focused tests;
- where a test intentionally keeps `mock_all_auths()`, verify `env.auths()` when the presence of an auth check matters;
- keep ordinary happy-path tests on `test_env()` when caller identity is not part of the assertion.

The goal is not to remove `mock_all_auths()` from every test. The goal is to ensure that each authorization boundary has dedicated coverage that can fail when the corresponding `require_auth()` rule is removed or changed.
