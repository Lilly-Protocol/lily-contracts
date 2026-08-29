#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

printf "rustc: "
rustc --version
printf "cargo: "
cargo --version

# Extract soroban-sdk major version from workspace
SDK_MAJOR=""
if [ -f "$REPO_ROOT/Cargo.lock" ]; then
  SDK_MAJOR="$(sed -n '/name = "soroban-sdk"/{n;s/version = "\([0-9]*\).*/\1/p;}' "$REPO_ROOT/Cargo.lock" | head -n 1)"
fi
if [ -z "$SDK_MAJOR" ] && [ -f "$REPO_ROOT/Cargo.toml" ]; then
  SDK_MAJOR="$(grep -E '^[[:space:]]*soroban-sdk[[:space:]]*=' "$REPO_ROOT/Cargo.toml" | sed -E 's/.*"([0-9]+).*/\1/' | head -n 1)"
fi
SDK_MAJOR="${SDK_MAJOR:-22}"

if command -v stellar >/dev/null 2>&1; then
  STELLAR_VERSION_RAW="$(stellar --version)"
  printf "stellar: %s\n" "$STELLAR_VERSION_RAW"

  CLI_MAJOR="$(printf "%s\n" "$STELLAR_VERSION_RAW" | sed -E 's/^[^0-9]*([0-9]+)\..*/\1/')"
  if [ -n "$CLI_MAJOR" ]; then
    if [ "$CLI_MAJOR" != "$SDK_MAJOR" ]; then
      printf "stellar / soroban-sdk compatibility: mismatch (CLI major %s != SDK major %s)\n" "$CLI_MAJOR" "$SDK_MAJOR" >&2
      printf "Error: stellar CLI major version (%s) is incompatible with soroban-sdk (%s)\n" "$CLI_MAJOR" "$SDK_MAJOR" >&2
      exit 1
    else
      printf "stellar / soroban-sdk compatibility: ok (CLI major %s matches SDK major %s)\n" "$CLI_MAJOR" "$SDK_MAJOR"
    fi
  else
    printf "stellar / soroban-sdk compatibility: warning (unable to parse CLI version from '%s')\n" "$STELLAR_VERSION_RAW"
  fi
else
  printf "stellar: not installed\n"
  if [ "${REQUIRE_STELLAR:-0}" = "1" ]; then
    printf "Error: stellar CLI is required but not installed\n" >&2
    exit 1
  fi
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
