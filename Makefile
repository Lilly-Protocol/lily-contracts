SHELL := /bin/sh

CONTRACT_PACKAGES := protocol identity wallet payments
WASM_TARGET := wasm32v1-none
ARTIFACTS_DIR := dist

# Every dependency-resolving cargo invocation uses --locked because
# Cargo.lock is committed: a stale lockfile must fail the build instead of
# silently drifting. (`cargo fmt` cannot take --locked; it only reads source
# files and never links dependencies, so it is the documented exception.)

.PHONY: fmt fmt-check lint check test test-locked audit docs size-report build build-wasm artifacts ci clean help

help:
	@printf "%s\n" \
	"make fmt         - format the workspace" \
	"make fmt-check   - verify formatting" \
	"make lint        - run clippy with warnings denied" \
	"make check       - cargo check across the workspace" \
	"make test        - run all unit and integration-style tests (locked)" \
	"make test-locked - cargo test --locked (alias for the locked test run)" \
	"make audit       - cargo audit (requires: cargo install cargo-audit)" \
	"make docs        - build rustdoc with warnings denied" \
	"make size-report - build Wasm artifacts and report their sizes" \
	"make build       - build the workspace" \
	"make build-wasm  - compile all contract packages to Wasm" \
	"make artifacts   - copy optimized Wasm artifacts into dist/" \
	"make ci          - local CI bundle (fmt-check, lint, test, docs)" \
	"make clean       - remove build outputs"

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --locked -- -D warnings

check:
	cargo check --workspace --locked

test:
	cargo test --workspace --locked

test-locked:
	cargo test --workspace --locked

audit:
	@command -v cargo-audit >/dev/null 2>&1 || { \
		echo "cargo-audit is not installed. Run: cargo install cargo-audit"; \
		exit 1; \
	}
	@ok=0; \
	for i in 1 2 3; do \
		cargo audit && { ok=1; break; }; \
		echo "audit: attempt $$i failed (advisory-db fetch?), retrying..."; \
		sleep 2; \
	done; \
	[ "$$ok" -eq 1 ]

docs:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

build:
	cargo build --workspace --locked

build-wasm:
	@test -d "$$(rustc --print sysroot)/lib/rustlib/$(WASM_TARGET)/lib" || { \
		echo "$(WASM_TARGET) target stdlib is not installed. Run: rustup target add $(WASM_TARGET)"; \
		exit 1; \
	}
	@for pkg in $(CONTRACT_PACKAGES); do \
		cargo build --locked --target $(WASM_TARGET) --profile release --package $$pkg; \
	done

artifacts: build-wasm
	@mkdir -p $(ARTIFACTS_DIR)
	@for pkg in $(CONTRACT_PACKAGES); do \
		cp target/$(WASM_TARGET)/release/$$pkg.wasm $(ARTIFACTS_DIR)/$$pkg.wasm; \
	done

size-report: artifacts
	@echo "Wasm artifact sizes:"; \
	for pkg in $(CONTRACT_PACKAGES); do \
		printf "  %-12s %10s bytes\n" "$$pkg" "$$(wc -c < $(ARTIFACTS_DIR)/$$pkg.wasm | tr -d ' ')"; \
	done

ci: fmt-check lint test docs

clean:
	rm -rf target $(ARTIFACTS_DIR)
