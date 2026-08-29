#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TMP_BIN_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_BIN_DIR"
}
trap cleanup EXIT

# Construct a clean PATH that excludes any pre-existing stellar CLI
CLEAN_PATH="$(printf "%s" "$PATH" | tr ':' '\n' | grep -v 'stellar' | tr '\n' ':' | sed 's/:$//')"

echo "=== Running check-tooling.sh tests ==="

# Test 1: When stellar is not in PATH and REQUIRE_STELLAR is not set, should succeed
echo "[Test 1] Uninstalled stellar default behaviour..."
OUTPUT="$(PATH="$CLEAN_PATH" "$REPO_ROOT/scripts/check-tooling.sh")"
echo "$OUTPUT" | grep -q "stellar: not installed"
echo "  Passed: uninstalled stellar handled gracefully."

# Test 2: When stellar is not in PATH and REQUIRE_STELLAR=1, should fail with non-zero exit code
echo "[Test 2] Uninstalled stellar with REQUIRE_STELLAR=1..."
if PATH="$CLEAN_PATH" REQUIRE_STELLAR=1 "$REPO_ROOT/scripts/check-tooling.sh" >/dev/null 2>&1; then
  echo "  Failed: expected exit code 1 when REQUIRE_STELLAR=1" >&2
  exit 1
else
  echo "  Passed: rejected missing stellar when REQUIRE_STELLAR=1."
fi

# Test 3: Compatible stellar version 22.8.2
echo "[Test 3] Compatible stellar CLI v22.8.2..."
cat << 'EOF' > "$TMP_BIN_DIR/stellar"
#!/usr/bin/env sh
echo "stellar 22.8.2 (7ab5f0565ed3a03b41789aaac7211c286fa0028c)"
EOF
chmod +x "$TMP_BIN_DIR/stellar"

OUTPUT="$(PATH="$TMP_BIN_DIR:$CLEAN_PATH" "$REPO_ROOT/scripts/check-tooling.sh")"
echo "$OUTPUT" | grep -q "stellar / soroban-sdk compatibility: ok (CLI major 22 matches SDK major 22)"
echo "  Passed: recognized and validated compatible CLI v22.8.2."

# Test 4: Compatible stellar-cli alternative format v22.0.1
echo "[Test 4] Compatible stellar CLI format v22.0.1..."
cat << 'EOF' > "$TMP_BIN_DIR/stellar"
#!/usr/bin/env sh
echo "stellar-cli 22.0.1"
EOF
chmod +x "$TMP_BIN_DIR/stellar"

OUTPUT="$(PATH="$TMP_BIN_DIR:$CLEAN_PATH" "$REPO_ROOT/scripts/check-tooling.sh")"
echo "$OUTPUT" | grep -q "stellar / soroban-sdk compatibility: ok (CLI major 22 matches SDK major 22)"
echo "  Passed: recognized and validated compatible CLI format v22.0.1."

# Test 5: Incompatible stellar version 23.0.1
echo "[Test 5] Incompatible stellar CLI v23.0.1..."
cat << 'EOF' > "$TMP_BIN_DIR/stellar"
#!/usr/bin/env sh
echo "stellar 23.0.1 (c4608c0 2025-08-11)"
EOF
chmod +x "$TMP_BIN_DIR/stellar"

if PATH="$TMP_BIN_DIR:$CLEAN_PATH" "$REPO_ROOT/scripts/check-tooling.sh" >/dev/null 2>&1; then
  echo "  Failed: expected exit code 1 for mismatched version v23.0.1" >&2
  exit 1
else
  echo "  Passed: rejected incompatible CLI v23.0.1."
fi

# Test 6: Incompatible stellar version 27.1.0
echo "[Test 6] Incompatible stellar CLI v27.1.0..."
cat << 'EOF' > "$TMP_BIN_DIR/stellar"
#!/usr/bin/env sh
echo "stellar 27.1.0"
EOF
chmod +x "$TMP_BIN_DIR/stellar"

if PATH="$TMP_BIN_DIR:$CLEAN_PATH" "$REPO_ROOT/scripts/check-tooling.sh" >/dev/null 2>&1; then
  echo "  Failed: expected exit code 1 for mismatched version v27.1.0" >&2
  exit 1
else
  echo "  Passed: rejected incompatible CLI v27.1.0."
fi

# Test 7: Running check-tooling.sh from a different working directory
echo "[Test 7] Running from different working directory..."
cat << 'EOF' > "$TMP_BIN_DIR/stellar"
#!/usr/bin/env sh
echo "stellar 22.8.2"
EOF
chmod +x "$TMP_BIN_DIR/stellar"

(cd "$TMP_BIN_DIR" && PATH="$TMP_BIN_DIR:$CLEAN_PATH" "$REPO_ROOT/scripts/check-tooling.sh") >/dev/null
echo "  Passed: script executes correctly from outside repo root."

echo "=== All check-tooling.sh tests passed! ==="
