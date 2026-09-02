# Security Policy

## Supported scope

This repository contains smart contract infrastructure and supporting contributor workflows for Lily Protocol on Stellar. Security-sensitive areas include:

- Contract authorization logic
- Storage layouts and upgrade assumptions
- Settlement state transitions
- Admin and initialization paths
- Build and deployment workflows
- Reentrancy guard semantics in `lily-common`

## Reentrancy guard

State-transition functions that mutate settlement state (currently
`payments::settle_intent` and `payments::cancel_intent`) hold a shared
`lily_common::NonReentrantGuard` across their mutation window.

- **Semantics.** The guard sets an instance-storage flag on acquire and clears
it on scope exit (including panic unwind). A second acquire of the same key
in the active window raises `ProtocolError::ReentrantCall`.
- **Layered defense.** The Soroban 22 host independently rejects re-invocation
of a contract instance that is already on the call stack. The guard
therefore covers the residual cases — recursive acquisition within one
frame, and SDK builds that permit reentry — and gives integrators a stable
typed error to match on.
- **Key discipline.** Each transition uses a distinct `Symbol` key (e.g.
`symbol_short!("settle")`) so guarded windows never collide with business
storage keys.
- **New transitions.** Any function that performs a non-idempotent state
transition should acquire the guard for the duration of its mutation block.

## Reporting a vulnerability

Please do not file public GitHub issues for vulnerabilities that could put funds, permissions, or protocol integrity at risk.

Report security issues privately to:

- `security@lilyprotocol.com`

Include:

- Affected contract or crate
- Impact summary
- Reproduction steps or proof of concept
- Suggested mitigations if known

We will acknowledge receipt as quickly as possible and coordinate next steps privately.
