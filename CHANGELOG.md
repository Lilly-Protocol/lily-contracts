# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Protocol configuration contract with one-time initialization, admin transfer,
  fee basis points, and treasury management.
- Agent identity registry with controller rotation, metadata updates, and
  administrative deactivation.
- Wallet policy registry with agent bindings, settlement asset configuration,
  spend limits, and enabled-state controls.
- Payment intent contract with payer cancellation and admin-driven settlement
  finalization.
- Shared protocol errors, payment status types, basis-point validation, and
  storage TTL helpers in `lily-common`.
- Reusable Soroban test helpers in `lily-test-support` and contract test suites
  covering initialization, authorization, storage, and state transitions.
- Formatting, linting, build, test, Wasm artifact, and CI workflows for the
  Rust workspace.

