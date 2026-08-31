#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

printf "rustc: "
rustc --version
printf "cargo: "
cargo --version

# Extract soroban-sdk major version from Cargo.toml or Cargo.lock
SDK_MAJOR=""
if [ -f "$REPO_ROOT/Cargo.toml" ]; then
  SDK_MAJOR=$(grep -E "^[[:space:]]*soroban-sdk[[:space:]]*=" "$REPO_ROOT/Cargo.toml" | head -n 1 | grep -oE "[0-9]+" | head -n 1)
fi
if [ -z "$SDK_MAJOR" ] && [ -f "$REPO_ROOT/Cargo.lock" ]; then
  SDK_MAJOR=$(grep -A 1 'name = "soroban-sdk"' "$REPO_ROOT/Cargo.lock" | grep "version =" | head -n 1 | grep -oE "[0-9]+" | head -n 1)
fi

if command -v stellar >/dev/null 2>&1; then
  printf "stellar: "
  STELLAR_RAW=$(stellar --version)
  printf "%s\n" "$STELLAR_RAW"

  # Validate CLI/SDK major version compatibility
  STELLAR_MAJOR=$(printf "%s\n" "$STELLAR_RAW" | grep -oE "[0-9]+\.[0-9]+(\.[0-9]+)?" | head -n 1 | cut -d. -f1)
  if [ -n "$SDK_MAJOR" ] && [ -n "$STELLAR_MAJOR" ]; then
    if [ "$STELLAR_MAJOR" = "$SDK_MAJOR" ]; then
      printf "stellar-cli major version matches soroban-sdk (%s): yes\n" "$SDK_MAJOR"
    else
      printf "ERROR: stellar-cli major version (%s) does not match soroban-sdk (%s)\n" "$STELLAR_MAJOR" "$SDK_MAJOR" >&2
      exit 1
    fi
  fi
else
  printf "stellar: not installed\n"
fi

if rustc --print target-list | grep -qx "wasm32v1-none"; then
  printf "wasm target available in toolchain list: yes\n"
else
  printf "wasm target available in toolchain list: no\n"
fi

if [ -d "$(rustc --print sysroot)/lib/rustlib/wasm32v1-none/lib" ]; then
  printf "wasm target stdlib installed: yes\n"
else
  printf "wasm target stdlib installed: no\n"
fi
