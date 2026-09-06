# Testing guide

All tests are plain Rust unit tests built with the `testutils` feature of
`soroban-sdk`; no network, snapshot, or external tooling is required.

```sh
make test             # cargo test --workspace
```

## Authorization mocking

Tests that need precise per-address authorization use
`env.mock_auths(&[MockAuth { address, invoke, sub_invokes }])` and re-arm the
exact entry before every client call. Contract panics carry the error enum and
are asserted with `#[should_panic(expected = "Error(Contract, #n)")]`.

## Property tests

#38 introduces proptest-based property coverage for unbounded, user-supplied
`String` inputs:

| Corpus (labels) | Crate | Strategy space |
| --- | --- | --- |
| identity URIs  | `contracts/identity` | `len: 1..=4096` chars, `seed: u64` |
| payment memos  | `contracts/payments` | `len: 0..=4096` chars, `seed: u64` |

Generated values mix 1-, 2-, 3- and 4-byte UTF-8 characters (including
emoji-width 4-byte sequences). Proptest is enabled per-crate through the
`attr-macro` feature (`proptest` in `[dev-dependencies]`); failing cases are
captured under the crate's `proptest-regressions/` corpus directory for
shrinking and replay, which is where the corpus is documented and persisted.

Assertions guarantee only the intended validation errors occur: long/multibyte
values must round-trip verbatim through the stored record, and the only
permitted failure is `ProtocolError::InvalidInput` (empty input), which is
covered deterministically by the `should_panic` tests.
