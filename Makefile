SHELL := /bin/sh

CONTRACT_PACKAGES := protocol identity wallet payments
WASM_TARGET := wasm32v1-none
ARTIFACTS_DIR := dist

.PHONY: fmt fmt-check lint check test test-locked audit docs size-report build build-wasm artifacts ci clean help

help:
	@printf "%s\n" \
	"make fmt         - format the workspace" \
	"make fmt-check   - verify formatting" \
	"make lint        - run clippy with warnings denied" \
	"make check       - cargo check across the workspace" \
	"make test        - run all unit and integration-style tests" \
	"make test-locked - run tests with locked dependencies" \
	"make audit       - run security vulnerability audit on dependencies" \
	"make docs        - build documentation without dependency docs" \
	"make size-report - report Wasm binary artifact sizes" \
	"make build       - build the workspace" \
	"make build-wasm  - compile all contract packages to Wasm" \
	"make artifacts   - copy optimized Wasm artifacts into dist/" \
	"make ci          - local CI bundle (fmt-check, lint, test)" \
	"make clean       - remove build outputs"

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

check:
	cargo check --workspace

test:
	cargo test --workspace

test-locked:
	cargo test --workspace --locked

audit:
	cargo audit

docs:
	cargo doc --workspace --no-deps

size-report: build-wasm
	@echo "=== Wasm Artifact Size Report ==="
	@for pkg in $(CONTRACT_PACKAGES); do \
		if [ -f target/$(WASM_TARGET)/release/$$pkg.wasm ]; then \
			ls -lh target/$(WASM_TARGET)/release/$$pkg.wasm | awk '{print $$9, ":", $$5}'; \
		fi \
	done

build:
	cargo build --workspace

build-wasm:
	@test -d "$$(rustc --print sysroot)/lib/rustlib/$(WASM_TARGET)/lib" || { \
		echo "$(WASM_TARGET) target stdlib is not installed. Run: rustup target add $(WASM_TARGET)"; \
		exit 1; \
	}
	@for pkg in $(CONTRACT_PACKAGES); do \
		cargo build --target $(WASM_TARGET) --profile release --package $$pkg; \
	done

artifacts: build-wasm
	@mkdir -p $(ARTIFACTS_DIR)
	@for pkg in $(CONTRACT_PACKAGES); do \
		cp target/$(WASM_TARGET)/release/$$pkg.wasm $(ARTIFACTS_DIR)/$$pkg.wasm; \
	done

ci: fmt-check lint test

clean:
	rm -rf target $(ARTIFACTS_DIR)
