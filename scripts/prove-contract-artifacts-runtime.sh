#!/usr/bin/env sh
# Runtime proof that dist/ matches what CI uploads as contract-artifacts.
# Run after `make artifacts` (requires dist/*.wasm and dist/manifest.json).
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"
MANIFEST="${ARTIFACTS_DIR}/manifest.json"

export ARTIFACTS_DIR

cd "$REPO_ROOT"
"$REPO_ROOT/scripts/verify-dist-manifest.sh"
"$REPO_ROOT/scripts/assert-contract-artifacts-bundle.sh"

artifact_count="$(jq '.artifacts | length' "$MANIFEST")"
commit="$(jq -r '.commit' "$MANIFEST" | cut -c1-12)"
profile="$(jq -r '.build_profile' "$MANIFEST")"

echo "Runtime proof passed: contract-artifacts bundle ready (${artifact_count} wasm + manifest.json)."
echo "  artifacts_dir=${ARTIFACTS_DIR} commit=${commit} profile=${profile}"
