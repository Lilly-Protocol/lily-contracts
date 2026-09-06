#!/usr/bin/env sh
# Tooling report.
#
#   check-tooling.sh           informational: print versions, always exit 0
#   check-tooling.sh --strict  fail (exit 1) if a required tool is missing
#
# Required (strict) tools: rustc, cargo, rustfmt, stellar, wasm32v1-none stdlib.
set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"

STRICT=0
for arg in "$@"; do
  case "$arg" in
    --strict)
      STRICT=1
      ;;
  esac
done

if [ "${CHECK_TOOLING_STRICT:-0}" = "1" ]; then
  STRICT=1
fi

# Derive repo root from the script's own directory so the script works even
# when REPO_ROOT is not exported by the caller.
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

missing_count=0

if command -v rustc >/dev/null 2>&1; then
  printf "rustc: "
  rustc --version
else
  printf "rustc: not installed\n"
  missing_count=$((missing_count + 1))
fi

if command -v cargo >/dev/null 2>&1; then
  printf "cargo: "
  cargo --version
else
  printf "cargo: not installed\n"
  missing_count=$((missing_count + 1))
fi

if command -v rustfmt >/dev/null 2>&1; then
  printf "rustfmt: "
  rustfmt --version
else
  printf "rustfmt: not installed\n"
  missing_count=$((missing_count + 1))
fi

# Extract soroban-sdk major version: prefer Cargo.lock, fall back to Cargo.toml
SDK_MAJOR=""
if [ -f "$REPO_ROOT/Cargo.lock" ]; then
  SDK_MAJOR="$(sed -n '/name = "soroban-sdk"/{n;s/version = "\([0-9]*\).*/\1/p;}' "$REPO_ROOT/Cargo.lock" | head -n 1)"
fi
if [ -z "$SDK_MAJOR" ] && [ -f "$REPO_ROOT/Cargo.toml" ]; then
  SDK_MAJOR="$(grep -E '^[[:space:]]*soroban-sdk[[:space:]]*=' "$REPO_ROOT/Cargo.toml" | sed -E 's/.*"([0-9]+).*/\1/' | head -n 1)"
fi
SDK_MAJOR="${SDK_MAJOR:-22}"

if command -v stellar >/dev/null 2>&1; then
  printf "stellar: "
  stellar --version
else
  printf "stellar: not installed\n"
  if [ "${REQUIRE_STELLAR:-0}" = "1" ]; then
    missing_count=$((missing_count + 1))
  fi
fi

if command -v rustc >/dev/null 2>&1 && rustc --print target-list | grep -qx "wasm32v1-none"; then
  printf "wasm target available in toolchain list: yes\n"
else
  printf "wasm32v1-none-target: not in toolchain target list\n"
  missing_count=$((missing_count + 1))
fi

if command -v rustc >/dev/null 2>&1 && [ -d "$(rustc --print sysroot)/lib/rustlib/wasm32v1-none/lib" ]; then
  printf "wasm target stdlib installed: yes\n"
else
  printf "wasm target stdlib installed: no\n"
  missing_count=$((missing_count + 1))
fi

if [ "${REQUIRE_STELLAR:-0}" = "1" ] && ! command -v stellar >/dev/null 2>&1; then
  exit 1
fi

if [ "$STRICT" = "1" ] && [ "$missing_count" -gt 0 ]; then
  printf "\nError: %d required tool(s) missing in strict mode.\n" "$missing_count" >&2
  exit 1
fi
