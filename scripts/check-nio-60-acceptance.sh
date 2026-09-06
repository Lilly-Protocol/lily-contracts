#!/usr/bin/env sh
# NIO-60 acceptance checklist — runnable without Rust/cargo.
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cd "$REPO_ROOT"

echo "NIO-60 acceptance checklist (manifest generator bounty)"
echo "======================================================"

# [1] Runtime manifest generation (same path as make artifacts post-copy)
echo "[1] Runtime manifest generation..."
ACCEPTANCE_DIST="$TMP_DIR/acceptance-dist"
mkdir -p "$ACCEPTANCE_DIST"
for pkg in protocol identity wallet payments; do
  printf '%s-acceptance-wasm' "$pkg" > "$ACCEPTANCE_DIST/${pkg}.wasm"
done
ARTIFACTS_DIR="$ACCEPTANCE_DIST" BUILD_PROFILE=release ./scripts/generate-manifest.sh
MANIFEST="$ACCEPTANCE_DIST/manifest.json"
test -f "$MANIFEST"
for field in commit build_profile version; do
  value="$(jq -r --arg f "$field" '.[$f] // empty' "$MANIFEST")"
  if [ -z "$value" ] || [ "$value" = "null" ]; then
    echo "FAIL (missing manifest field: $field)" >&2
    exit 1
  fi
done
jq -e '.artifacts | length == 4' "$MANIFEST" >/dev/null
jq -e '.artifacts | first | .sha256 | length > 0' "$MANIFEST" >/dev/null
echo "    PASS (dist/manifest.json generated with hashes, version, commit, profile)"

# [2] Behavioral test suite
echo "[2] Manifest field + bundle validation tests..."
./scripts/test-generate-manifest.sh
echo "    PASS (test-generate-manifest.sh)"

# [3] CI wiring (static) + runtime contract-artifacts bundle proof
echo "[3] CI artifacts include manifest..."
./scripts/validate-artifacts-ci-wiring.sh
echo "    PASS (CI/release/Makefile wiring)"
ARTIFACTS_DIR="$ACCEPTANCE_DIST" ./scripts/prove-contract-artifacts-runtime.sh
echo "    PASS (runtime contract-artifacts bundle proof on generated dist/)"

echo ""
echo "Out of bounty scope (Rowan decision required for full QA PASS):"
if command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
  if make artifacts >/dev/null 2>&1; then
    echo "  contract Wasm e2e: PASS (make artifacts succeeded)"
  else
    echo "  contract Wasm e2e: BLOCKED (make artifacts failed — workspace health)"
  fi
else
  echo "  contract Wasm e2e: SKIPPED (cargo/rustc not installed)"
fi
echo "  green CI upload evidence: BLOCKED (requires push to remote)"

echo ""
echo "NIO-60 bounty acceptance criteria: PASS (items 1–3, runtime-verified)."
