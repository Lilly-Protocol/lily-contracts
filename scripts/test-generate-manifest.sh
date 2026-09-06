#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

assert_json_field() {
  file="$1"
  jq_expr="$2"
  expected="$3"
  actual="$(jq -r "$jq_expr" "$file")"
  if [ "$actual" != "$expected" ]; then
    echo "  Failed: expected $jq_expr = '$expected', got '$actual'" >&2
    exit 1
  fi
}

echo "=== Running generate-manifest.sh tests ==="

# Test 1: generates manifest with hashes, version, commit, and profile
echo "[Test 1] Manifest includes required provenance fields..."
ARTIFACTS_DIR="$TMP_DIR/dist"
mkdir -p "$ARTIFACTS_DIR"
printf 'protocol-wasm-bytes' > "$ARTIFACTS_DIR/protocol.wasm"
printf 'identity-wasm-bytes' > "$ARTIFACTS_DIR/identity.wasm"

(
  cd "$REPO_ROOT"
  ARTIFACTS_DIR="$ARTIFACTS_DIR" BUILD_PROFILE=release ./scripts/generate-manifest.sh
)

MANIFEST="$ARTIFACTS_DIR/manifest.json"
[ -f "$MANIFEST" ]

assert_json_field "$MANIFEST" '.version' '0.1.0'
assert_json_field "$MANIFEST" '.build_profile' 'release'
assert_json_field "$MANIFEST" '.commit' "$(git -C "$REPO_ROOT" rev-parse HEAD)"
assert_json_field "$MANIFEST" '.artifacts | length' '2'

protocol_hash="$(sha256sum "$ARTIFACTS_DIR/protocol.wasm" | awk '{print $1}')"
identity_hash="$(sha256sum "$ARTIFACTS_DIR/identity.wasm" | awk '{print $1}')"
assert_json_field "$MANIFEST" '.artifacts[] | select(.package == "protocol") | .sha256' "$protocol_hash"
assert_json_field "$MANIFEST" '.artifacts[] | select(.package == "identity") | .profile' 'release'
assert_json_field "$MANIFEST" '.artifacts[] | select(.package == "identity") | .sha256' "$identity_hash"
echo "  Passed: manifest contains hashes, version, commit, and profile."

# Test 2: fails when artifacts directory is missing
echo "[Test 2] Missing artifacts directory is rejected..."
if ARTIFACTS_DIR="$TMP_DIR/missing-dist" "$REPO_ROOT/scripts/generate-manifest.sh" >/dev/null 2>&1; then
  echo "  Failed: expected non-zero exit for missing artifacts directory" >&2
  exit 1
fi
echo "  Passed: missing artifacts directory rejected."

# Test 3: respects custom build profile
echo "[Test 3] Custom BUILD_PROFILE is recorded..."
ARTIFACTS_DIR="$TMP_DIR/profile-dist"
mkdir -p "$ARTIFACTS_DIR"
printf 'wallet-wasm' > "$ARTIFACTS_DIR/wallet.wasm"

(
  cd "$REPO_ROOT"
  ARTIFACTS_DIR="$ARTIFACTS_DIR" BUILD_PROFILE=dev ./scripts/generate-manifest.sh
)

assert_json_field "$ARTIFACTS_DIR/manifest.json" '.build_profile' 'dev'
assert_json_field "$ARTIFACTS_DIR/manifest.json" '.artifacts[0].profile' 'dev'
echo "  Passed: custom build profile recorded."

# Test 4: artifacts are emitted in stable alphabetical order
echo "[Test 4] Artifact entries are sorted by package name..."
ARTIFACTS_DIR="$TMP_DIR/sort-dist"
mkdir -p "$ARTIFACTS_DIR"
printf 'z-last' > "$ARTIFACTS_DIR/wallet.wasm"
printf 'a-first' > "$ARTIFACTS_DIR/identity.wasm"
printf 'm-middle' > "$ARTIFACTS_DIR/payments.wasm"
printf 'b-second' > "$ARTIFACTS_DIR/protocol.wasm"

(
  cd "$REPO_ROOT"
  ARTIFACTS_DIR="$ARTIFACTS_DIR" ./scripts/generate-manifest.sh
)

package_order="$(jq -r '.artifacts[].package' "$ARTIFACTS_DIR/manifest.json" | tr '\n' ' ')"
expected_order="identity payments protocol wallet "
if [ "$package_order" != "$expected_order" ]; then
  echo "  Failed: expected order '$expected_order', got '$package_order'" >&2
  exit 1
fi
echo "  Passed: artifacts sorted alphabetically regardless of file creation order."

# Test 5: simulate `make artifacts` copy + manifest generation path
echo "[Test 5] Make artifacts integration path (copy dist + verify manifest)..."
INTEGRATION_ROOT="$TMP_DIR/integration"
WASM_DIR="$INTEGRATION_ROOT/target/wasm32v1-none/release"
DIST_DIR="$INTEGRATION_ROOT/dist"
mkdir -p "$WASM_DIR" "$DIST_DIR"

for pkg in protocol identity wallet payments; do
  printf '%s-wasm-payload' "$pkg" > "$WASM_DIR/${pkg}.wasm"
done

for pkg in protocol identity wallet payments; do
  cp "$WASM_DIR/${pkg}.wasm" "$DIST_DIR/${pkg}.wasm"
done

(
  cd "$REPO_ROOT"
  ARTIFACTS_DIR="$DIST_DIR" BUILD_PROFILE=release ./scripts/generate-manifest.sh
)

ARTIFACTS_DIR="$DIST_DIR" "$REPO_ROOT/scripts/verify-dist-manifest.sh"
ARTIFACTS_DIR="$DIST_DIR" "$REPO_ROOT/scripts/assert-contract-artifacts-bundle.sh"
echo "  Passed: dist manifest matches copied Wasm artifacts."

# Test 6: contract-artifacts bundle matches CI upload paths
echo "[Test 6] Contract-artifacts bundle paths..."
BUNDLE_DIR="$TMP_DIR/bundle-dist"
mkdir -p "$BUNDLE_DIR"
for pkg in protocol identity wallet payments; do
  printf '%s' "$pkg" > "$BUNDLE_DIR/${pkg}.wasm"
done
printf '{}' > "$BUNDLE_DIR/manifest.json"
ARTIFACTS_DIR="$BUNDLE_DIR" "$REPO_ROOT/scripts/assert-contract-artifacts-bundle.sh"
echo "  Passed: CI upload bundle paths are present."

# Test 7: smoke script rejects failed make artifacts without explicit fallback opt-in
echo "[Test 7] Smoke script fails without fallback when make artifacts unavailable..."
if PATH="/usr/bin:/bin" ARTIFACTS_SMOKE_ALLOW_FALLBACK=0 "$REPO_ROOT/scripts/test-artifacts-smoke.sh" >/dev/null 2>&1; then
  echo "  Failed: expected non-zero exit when cargo is unavailable" >&2
  exit 1
fi
echo "  Passed: smoke script fails closed without ARTIFACTS_SMOKE_ALLOW_FALLBACK."

# Test 8 (optional): offline manifest pipeline when rustc is available
if command -v rustc >/dev/null 2>&1; then
  echo "[Test 8] Offline manifest pipeline with rustc minimal Wasm..."
  OFFLINE_DIR="$TMP_DIR/offline-dist"
  mkdir -p "$OFFLINE_DIR"
  ARTIFACTS_DIR="$OFFLINE_DIR" WASM_DIR="$TMP_DIR/offline-wasm/release" \
    "$REPO_ROOT/scripts/test-artifacts-manifest-offline.sh"
  echo "  Passed: offline manifest pipeline with real Wasm modules."
else
  echo "[Test 8] Skipped: rustc not available for offline manifest pipeline."
fi

# Test 9: CI/release wiring for contract-artifacts (no Rust required)
echo "[Test 9] Artifacts CI wiring validation..."
"$REPO_ROOT/scripts/validate-artifacts-ci-wiring.sh"
echo "  Passed: CI/release workflows wire manifest into contract-artifacts."

# Test 10: runtime proof script on simulated dist bundle
echo "[Test 10] Runtime proof script on simulated dist..."
INTEGRATION_DIST="$TMP_DIR/runtime-proof-dist"
mkdir -p "$INTEGRATION_DIST"
for pkg in protocol identity wallet payments; do
  printf '%s-runtime-wasm' "$pkg" > "$INTEGRATION_DIST/${pkg}.wasm"
done
(
  cd "$REPO_ROOT"
  ARTIFACTS_DIR="$INTEGRATION_DIST" BUILD_PROFILE=release ./scripts/generate-manifest.sh
)
ARTIFACTS_DIR="$INTEGRATION_DIST" "$REPO_ROOT/scripts/prove-contract-artifacts-runtime.sh"
echo "  Passed: runtime proof script validates dist bundle."

# Test 11: prove script summary honors ARTIFACTS_DIR when dist/ also exists
echo "[Test 11] Prove script honors ARTIFACTS_DIR over stale dist/..."
ISOLATED_DIST="$TMP_DIR/isolated-proof-dist"
STALE_DIST="$TMP_DIR/stale-dist"
mkdir -p "$ISOLATED_DIST" "$STALE_DIST"
for pkg in protocol identity wallet payments; do
  printf 'isolated-%s' "$pkg" > "$ISOLATED_DIST/${pkg}.wasm"
  printf 'stale-%s' "$pkg" > "$STALE_DIST/${pkg}.wasm"
done
(
  cd "$REPO_ROOT"
  ARTIFACTS_DIR="$STALE_DIST" BUILD_PROFILE=dev ./scripts/generate-manifest.sh
  ARTIFACTS_DIR="$ISOLATED_DIST" BUILD_PROFILE=release ./scripts/generate-manifest.sh
)
proof_output="$(ARTIFACTS_DIR="$ISOLATED_DIST" "$REPO_ROOT/scripts/prove-contract-artifacts-runtime.sh")"
echo "$proof_output" | grep -q "artifacts_dir=$ISOLATED_DIST" || {
  echo "  Failed: prove script summary must reference ARTIFACTS_DIR" >&2
  exit 1
}
echo "$proof_output" | grep -q 'profile=release' || {
  echo "  Failed: prove script must read manifest from ARTIFACTS_DIR, not stale dist/" >&2
  exit 1
}
echo "  Passed: prove script is env-isolated from stale dist/."

# Test 12: Studio harness verify entry (package.json overrides cargo test detection)
echo "[Test 12] package.json harness verify wiring..."
if [ ! -f "$REPO_ROOT/package.json" ]; then
  echo "  Failed: package.json required for harness verify detection" >&2
  exit 1
fi
grep -qE '"test".*(verify\.sh|sh scripts/verify)' "$REPO_ROOT/package.json" || {
  echo "  Failed: package.json test script must invoke verify.sh" >&2
  exit 1
}
[ -x "$REPO_ROOT/scripts/verify.sh" ] || {
  echo "  Failed: scripts/verify.sh must be executable" >&2
  exit 1
}
echo "  Passed: package.json test script invokes verify.sh (run npm run test outside suite)."

echo "=== All generate-manifest.sh tests passed! ==="
