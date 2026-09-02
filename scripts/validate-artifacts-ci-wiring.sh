#!/usr/bin/env sh
# Static checks for manifest + contract-artifacts CI wiring (no Rust required).
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CI_FILE="$REPO_ROOT/.github/workflows/ci.yml"
RELEASE_FILE="$REPO_ROOT/.github/workflows/release.yml"
MAKEFILE="$REPO_ROOT/Makefile"

require_file() {
  if [ ! -f "$1" ]; then
    echo "Error: missing required file: $1" >&2
    exit 1
  fi
}

require_file "$CI_FILE"
require_file "$RELEASE_FILE"
require_file "$MAKEFILE"
require_file "$REPO_ROOT/scripts/generate-manifest.sh"
require_file "$REPO_ROOT/scripts/verify-dist-manifest.sh"
require_file "$REPO_ROOT/scripts/assert-contract-artifacts-bundle.sh"

require_file "$REPO_ROOT/scripts/prove-contract-artifacts-runtime.sh"
require_file "$REPO_ROOT/scripts/verify.sh"
require_file "$REPO_ROOT/package.json"

grep -qE '"test".*(verify\.sh|sh scripts/verify)' "$REPO_ROOT/package.json" || {
  echo "Error: package.json test script must invoke verify.sh for harness detection." >&2
  exit 1
}

grep -q 'run: make verify' "$CI_FILE" || {
  echo "Error: CI must run make verify (NIO-60 manifest acceptance)." >&2
  exit 1
}

grep -q 'run: make artifacts' "$CI_FILE" || {
  echo "Error: CI must run make artifacts." >&2
  exit 1
}

grep -q 'prove-contract-artifacts-runtime.sh' "$CI_FILE" || {
  echo "Error: CI must run prove-contract-artifacts-runtime.sh after make artifacts." >&2
  exit 1
}

grep -q 'verify-dist-manifest.sh' "$REPO_ROOT/scripts/prove-contract-artifacts-runtime.sh" || {
  echo "Error: prove script must run verify-dist-manifest.sh." >&2
  exit 1
}

grep -q 'assert-contract-artifacts-bundle.sh' "$REPO_ROOT/scripts/prove-contract-artifacts-runtime.sh" || {
  echo "Error: prove script must run assert-contract-artifacts-bundle.sh." >&2
  exit 1
}

grep -q 'dist/manifest.json' "$CI_FILE" || {
  echo "Error: CI upload must include dist/manifest.json." >&2
  exit 1
}

grep -q 'if-no-files-found: error' "$CI_FILE" || {
  echo "Error: CI artifact upload must fail when files are missing." >&2
  exit 1
}

if grep -q 'test-artifacts-smoke.sh' "$CI_FILE" && ! grep -q 'ARTIFACTS_SMOKE_ALLOW_FALLBACK' "$CI_FILE"; then
  :
elif grep -q 'test-artifacts-smoke.sh' "$CI_FILE"; then
  echo "Error: CI must not run smoke script with fallback enabled." >&2
  exit 1
fi

grep -q 'dist/manifest.json' "$RELEASE_FILE" || {
  echo "Error: release workflow must upload dist/manifest.json." >&2
  exit 1
}

grep -q 'generate-manifest.sh' "$MAKEFILE" || {
  echo "Error: Makefile artifacts target must invoke generate-manifest.sh." >&2
  exit 1
}

grep -q 'verify:' "$MAKEFILE" || {
  echo "Error: Makefile must define verify target for NIO-60 acceptance." >&2
  exit 1
}

# Upload step must appear after runtime proof.
ci_upload_line="$(grep -n 'Upload contract artifacts' "$CI_FILE" | head -1 | cut -d: -f1)"
ci_prove_line="$(grep -n 'prove-contract-artifacts-runtime.sh' "$CI_FILE" | head -1 | cut -d: -f1)"

if [ -z "$ci_prove_line" ] || [ "$ci_prove_line" -ge "$ci_upload_line" ]; then
  echo "Error: CI must prove dist bundle before uploading artifacts." >&2
  exit 1
fi

echo "Artifacts CI wiring validation passed."
