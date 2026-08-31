#!/usr/bin/env sh
set -eu

printf "rustc: "
rustc --version
printf "cargo: "
cargo --version

if command -v stellar >/dev/null 2>&1; then
  printf "stellar: "
  stellar --version
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

# Verify stellar-cli major.minor matches soroban-sdk major.minor.
if command -v stellar >/dev/null 2>&1 && [ -f Cargo.lock ]; then
  sdk_version=$(grep -A 1 'name = "soroban-sdk"' Cargo.lock | grep 'version = ' | sed 's/.*"\([0-9]*\.[0-9]*\).*/\1/')
  cli_version=$(stellar --version | sed -n 's/.*\([0-9]*\.[0-9]*\).*/\1/p')

  if [ -n "$sdk_version" ] && [ -n "$cli_version" ]; then
    if [ "$sdk_version" = "$cli_version" ]; then
      printf "stellar-cli / soroban-sdk compatibility: ok (%s)\n" "$sdk_version"
    else
      printf "stellar-cli / soroban-sdk compatibility: mismatch (cli %s vs sdk %s)\n" "$cli_version" "$sdk_version" >&2
      exit 1
    fi
  else
    printf "stellar-cli / soroban-sdk compatibility: could not determine\n"
  fi
fi
