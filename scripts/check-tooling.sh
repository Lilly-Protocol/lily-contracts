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

# --- soroban-sdk / stellar-cli compatibility gate ---------------------------
# A CLI/SDK major mismatch (e.g. CLI v23 against soroban-sdk 22) can
# produce incompatible contract spec output or deploy-tooling surprises.
# CI therefore requires the stellar-cli pin in .github/workflows/ci.yml
# to stay on the same major line as the SDK version locked in Cargo.lock.
if [ -f Cargo.lock ] && [ -f .github/workflows/ci.yml ]; then
  SDK_VERSION="$(awk '/^name = "soroban-sdk"$/{getline; sub(/^version = /,""); gsub(/"/,""); print; exit}' Cargo.lock)"
  CLI_MAJOR="$(awk -F@ '/stellar-cli@/{split($2,a,"."); gsub(/[^0-9]/,"",a[1]); print a[1]; exit}' .github/workflows/ci.yml)"
  if [ -n "$SDK_VERSION" ] && [ -n "$CLI_MAJOR" ]; then
    SDK_MAJOR="${SDK_VERSION%%.*}"
    if [ "$SDK_MAJOR" = "$CLI_MAJOR" ]; then
      printf "stellar-cli/SDK alignment: OK (CLI major %s ~= soroban-sdk %s)\n" "$CLI_MAJOR" "$SDK_VERSION"
    else
      {
        printf "ERROR: stellar-cli major %s (.github/workflows/ci.yml) does not match soroban-sdk %s major %s (Cargo.lock)\n" "$CLI_MAJOR" "$SDK_VERSION" "$SDK_MAJOR"
        printf "Repin to stellar/stellar-cli@v%s.x matching the SDK, or update the SDK after a deliberate CLI bump.\n" "$SDK_MAJOR"
      } >&2
      exit 1
    fi
  else
    printf "ERROR: cannot resolve soroban-sdk version (Cargo.lock) or stellar-cli pin (.github/workflows/ci.yml)\n" >&2
    exit 1
  fi
else
  printf "warn: running outside repo root (Cargo.lock / .github/workflows/ci.yml not found); skipping SDK/CLI gate\n"
fi
