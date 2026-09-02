#!/usr/bin/env sh
# Tooling report.
#
#   check-tooling.sh           informational: print versions, always exit 0
#   check-tooling.sh --strict  fail (exit 1) if a required tool is missing
#
# Required (strict) tools: rustc, cargo, rustfmt, stellar, wasm32v1-none stdlib.
set -u

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

if command -v stellar >/dev/null 2>&1; then
  have stellar stellar --version
else
  printf "stellar: not installed\n"
  missing_count=$((missing_count + 1))
fi

if command -v rustc >/dev/null 2>&1 && rustc --print target-list | grep -qx "wasm32v1-none"; then
  printf "wasm target available in toolchain list: yes\n"
else
  miss wasm32v1-none-target "not in toolchain target list"
fi

if command -v rustc >/dev/null 2>&1 && [ -d "$(rustc --print sysroot)/lib/rustlib/wasm32v1-none/lib" ]; then
  printf "wasm target stdlib installed: yes\n"
else
  printf "wasm target stdlib installed: no\n"
  missing_count=$((missing_count + 1))
fi

if [ "$STRICT" = "1" ] && [ "$missing_count" -gt 0 ]; then
  printf "\nError: %d required tool(s) missing in strict mode.\n" "$missing_count" >&2
  exit 1
fi

