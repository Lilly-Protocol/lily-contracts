#!/usr/bin/env sh
set -eu

printf "rustc: "
rustc --version
printf "cargo: "
cargo --version

if command -v stellar >/dev/null 2>&1; then
  printf "stellar: "
  STELLAR_VERSION=$(stellar --version)
  echo "$STELLAR_VERSION"
  
  if echo "$STELLAR_VERSION" | grep -q -E "^(stellar )?22\."; then
    printf "stellar-cli major version matches soroban-sdk (22): yes\n"
  else
    printf "ERROR: stellar-cli major version does not match soroban-sdk 22.\n"
    exit 1
  fi
else
  printf "stellar: not installed\n"
  exit 1
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
