#!/usr/bin/env sh
# End-to-end smoke: contract `make artifacts`, manifest verify, CI upload bundle.
# Fails when `make artifacts` fails unless ARTIFACTS_SMOKE_ALLOW_FALLBACK=1 (local only).
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"

if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
  echo "Error: cargo and rustc are required for artifacts smoke test." >&2
  exit 1
fi

cd "$REPO_ROOT"

if make artifacts; then
  echo "Built contract Wasm via make artifacts."
else
  if [ "${ARTIFACTS_SMOKE_ALLOW_FALLBACK:-0}" = "1" ]; then
    echo "make artifacts failed; running offline manifest fallback (local dev only)." >&2
    exec "$REPO_ROOT/scripts/test-artifacts-manifest-offline.sh"
  fi
  echo "Error: make artifacts failed. Set ARTIFACTS_SMOKE_ALLOW_FALLBACK=1 for offline manifest-only testing." >&2
  exit 1
fi

"$REPO_ROOT/scripts/verify-dist-manifest.sh"
"$REPO_ROOT/scripts/assert-contract-artifacts-bundle.sh"

echo "Artifacts smoke test passed (contract Wasm + manifest provenance)."
