SHELL := /bin/sh

CONTRACT_PACKAGES := protocol identity wallet payments
WASM_TARGET := wasm32v1-none
ARTIFACTS_DIR := dist

.PHONY: fmt fmt-check lint check test doc build build-wasm artifacts ci clean help

help:
	@printf "%s\n" \
	"make fmt        - format the workspace" \
	"make fmt-check  - verify formatting" \
	"make lint       - run clippy with warnings denied" \
	"make check      - cargo check across the workspace" \
	"make test       - run all unit and integration-style tests" \
	"make doc        - generate documentation with warnings denied" \
	"make build      - build the workspace" \
	"make build-wasm - compile all contract packages to Wasm (with size regression gate)" \
	"make wasm-size  - compile and check wasm sizes against the committed baseline" \
	"make artifacts  - copy optimized Wasm artifacts into dist/" \
	"make ci         - local CI bundle (fmt-check, lint, test, doc)" \
	"make clean      - remove build outputs"

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --locked --workspace --all-targets -- -D warnings

check:
	cargo check --locked --workspace

test:
	cargo test --locked --workspace

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

build:
	cargo build --locked --workspace

build-wasm:
	@test -d "$$(rustc --print sysroot)/lib/rustlib/$(WASM_TARGET)/lib" || { \
		echo "$(WASM_TARGET) target stdlib is not installed. Run: rustup target add $(WASM_TARGET)"; \
		exit 1; \
	}
	@for pkg in $(CONTRACT_PACKAGES); do \
		cargo build --locked --target $(WASM_TARGET) --profile release --package $$pkg; \
	done
	@sh scripts/check-wasm-size.sh

wasm-size: build-wasm
	@echo "wasm size regression gate: PASS"

artifacts: build-wasm
	@mkdir -p $(ARTIFACTS_DIR)
	@for pkg in $(CONTRACT_PACKAGES); do \
		cp target/$(WASM_TARGET)/release/$$pkg.wasm $(ARTIFACTS_DIR)/$$pkg.wasm; \
	done
	@./scripts/generate-manifest.sh

ci: fmt-check lint test doc

clean:
	rm -rf target $(ARTIFACTS_DIR)
