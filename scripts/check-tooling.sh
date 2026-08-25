#!/usr/bin/env sh
set -eu

strict=0
if [ "${1:-}" = "--strict" ]; then
  strict=1
fi

status=0

ok() {
  printf "%s\n" "$1"
}

missing() {
  printf "%s\n" "$1"
  status=1
}

if command -v rustc >/dev/null 2>&1; then
  printf "rustc: "
  rustc --version
else
  missing "rustc: not installed"
fi

if command -v cargo >/dev/null 2>&1; then
  printf "cargo: "
  cargo --version
else
  missing "cargo: not installed"
fi

if command -v rustfmt >/dev/null 2>&1 || cargo fmt --version >/dev/null 2>&1; then
  printf "rustfmt: "
  if command -v rustfmt >/dev/null 2>&1; then
    rustfmt --version
  else
    cargo fmt --version
  fi
else
  missing "rustfmt: not installed"
fi

if command -v stellar >/dev/null 2>&1; then
  printf "stellar: "
  stellar --version
else
  missing "stellar: not installed"
fi

if rustc --print target-list 2>/dev/null | grep -qx "wasm32v1-none"; then
  printf "wasm target available in toolchain list: yes\n"
else
  missing "wasm target available in toolchain list: no"
fi

if command -v rustc >/dev/null 2>&1 && [ -d "$(rustc --print sysroot)/lib/rustlib/wasm32v1-none/lib" ]; then
  printf "wasm target stdlib installed: yes\n"
else
  missing "wasm target stdlib installed: no"
fi

if [ "$strict" -eq 1 ]; then
  exit "$status"
fi

exit 0
