# Contract Testing

This repository's shared `test_env()` helper calls `Env::mock_all_auths()`.
That keeps behavior tests concise, but it also authorizes every
`Address::require_auth()` invocation automatically. A test that uses
`test_env()` therefore cannot prove that a contract rejects a missing or
incorrect authorization.

## Choosing an authorization setup

Use `test_env()` when authorization is not the behavior under test, including:

- storage and state-transition tests after a valid caller is assumed;
- validation of amounts, statuses, and initialization rules;
- event payload and query behavior; and
- multi-step happy paths where repeating explicit auth trees would obscure the
  protocol behavior being checked.

Do not use `test_env()` for authorization boundaries. Tests for admin-only,
controller-only, payer-only, or multi-party operations must start with
`Env::default()` and opt in to only the authorization being exercised.

## Negative authorization tests

The simplest missing-auth test uses an environment without auth mocking and
asserts that the client call fails:

```rust
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
#[should_panic]
fn rejects_missing_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    // No mock authorization is installed, so admin.require_auth() must fail.
    client.initialize(&admin, &treasury, &100_u32);
}
```

For a positive authorization-boundary test, prefer `Env::mock_auths()` with a
specific `MockAuth` invocation tree. That proves the call succeeds with the
expected signer without silently authorizing unrelated addresses or nested
calls. Keep the mocked function name, arguments, and sub-invocations aligned
with the contract call under test.

When an operation requires multiple actors, add separate cases for:

1. no authorization;
2. each required authorization missing in turn;
3. an unrelated address authorizing the call; and
4. the complete expected authorization set.

## Current coverage debt

The existing protocol, identity, wallet, and payments suites all construct
their environment through `test_env()`. Their state and validation assertions
remain useful, but their successful calls do not currently verify real auth
boundaries.

Close this gap incrementally:

1. Add one missing-auth test for every public function that calls
   `require_auth()`.
2. Add wrong-signer tests for role-specific functions such as admin,
   controller, agent, wallet, and payer operations.
3. Add explicit positive `mock_auths()` tests for multi-party and nested
   authorization trees.
4. Keep broad `mock_all_auths()` behavior tests only where auth is outside the
   test's stated purpose.

New or changed authorization paths should include these focused negative tests
in the same pull request. A passing all-mock suite alone is not evidence that an
authorization boundary is enforced.

