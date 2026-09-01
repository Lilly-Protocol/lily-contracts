#!/usr/bin/env sh
# Tooling report.
#
#   check-tooling.sh           informational: print versions, always exit 0
#   check-tooling.sh --strict  fail (exit 1) if a required tool is missing
#
# Required (strict) tools: rustc, cargo, rustfmt, stellar, wasm32v1-none stdlib.
set -u

STRICT=0
if [ "${1:-}" = "--strict" ]; then
  STRICT=1
fi
missing=0

have() {
  # Report a tool that resolved successfully.
  printf "%s: " "$1"
  shift
  "$@"
}

miss() {
  # Report a missing tool; count it (the exit code decides on STRICT).
  if [ "$STRICT" -eq 1 ]; then
    printf "%s: %s (REQUIRED)\n" "$1" "$2"
  else
    printf "%s: %s\n" "$1" "$2"
  fi
  missing=$((missing + 1))
}

if command -v rustc >/dev/null 2>&1; then
  have rustc rustc --version
else
  miss rustc "not installed"
fi

if command -v cargo >/dev/null 2>&1; then
  have cargo cargo --version
else
  miss cargo "not installed"
fi

if cargo fmt --version >/dev/null 2>&1; then
  have rustfmt cargo fmt --version
else
  miss rustfmt "not installed"
fi

if command -v stellar >/dev/null 2>&1; then
  have stellar stellar --version
else
  miss stellar "not installed"
fi

if rustc --print target-list 2>/dev/null | grep -qx "wasm32v1-none"; then
  printf "wasm32v1-none in toolchain target list: yes\n"
else
  miss wasm32v1-none-target "not in toolchain target list"
fi

if [ -d "$(rustc --print sysroot 2>/dev/null)/lib/rustlib/wasm32v1-none/lib" ]; then
  printf "wasm32v1-none stdlib installed: yes\n"
else
  miss wasm32v1-none-stdlib "not installed (run: rustup target add wasm32v1-none)"
fi

case "$STRICT:$missing" in
  1:0)
    printf "check-tooling: OK (strict, all required tools present)\n"
    exit 0
    ;;
  1:*)
    printf "check-tooling: FAILED (strict, %d required tool(s) missing)\n" "$missing"
    exit 1
    ;;
  *)
    printf "check-tooling: OK (informational, %d tool(s) not installed)\n" "$missing"
    exit 0
    ;;
esac
