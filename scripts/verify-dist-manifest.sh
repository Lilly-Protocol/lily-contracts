#!/usr/bin/env sh
# Validate dist/manifest.json against dist/*.wasm after `make artifacts`.
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"
MANIFEST="${ARTIFACTS_DIR}/manifest.json"
EXPECTED_PACKAGES="${EXPECTED_PACKAGES:-protocol identity wallet payments}"

if [ ! -f "$MANIFEST" ]; then
  echo "Error: manifest not found at '$MANIFEST' (run make artifacts first)." >&2
  exit 1
fi

if ! jq empty "$MANIFEST" >/dev/null 2>&1; then
  echo "Error: '$MANIFEST' is not valid JSON." >&2
  exit 1
fi

for field in commit build_profile version; do
  value="$(jq -r --arg f "$field" '.[$f] // empty' "$MANIFEST")"
  if [ -z "$value" ] || [ "$value" = "null" ]; then
    echo "Error: manifest missing required field '$field'." >&2
    exit 1
  fi
done

if [ "$(jq -r '.commit' "$MANIFEST")" = "unknown" ]; then
  echo "Error: manifest commit must not be 'unknown' in CI builds." >&2
  exit 1
fi

artifact_count="$(jq '.artifacts | length' "$MANIFEST")"
if [ "$artifact_count" -eq 0 ]; then
  echo "Error: manifest contains no artifacts." >&2
  exit 1
fi

packages="$(jq -r '.artifacts[].package' "$MANIFEST")"
sorted_packages="$(printf "%s\n" "$packages" | sort)"
if [ "$packages" != "$sorted_packages" ]; then
  echo "Error: manifest artifacts are not sorted by package name." >&2
  printf "  order: %s\n" "$packages" >&2
  exit 1
fi

for pkg in $EXPECTED_PACKAGES; do
  wasm_file="${ARTIFACTS_DIR}/${pkg}.wasm"
  if [ ! -f "$wasm_file" ]; then
    echo "Error: expected Wasm artifact missing: $wasm_file" >&2
    exit 1
  fi

  manifest_hash="$(jq -r --arg pkg "$pkg" '.artifacts[] | select(.package == $pkg) | .sha256' "$MANIFEST")"
  if [ -z "$manifest_hash" ] || [ "$manifest_hash" = "null" ]; then
    echo "Error: manifest missing entry for package '$pkg'." >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    actual_hash="$(sha256sum "$wasm_file" | awk '{print $1}')"
  else
    actual_hash="$(shasum -a 256 "$wasm_file" | awk '{print $1}')"
  fi

  if [ "$manifest_hash" != "$actual_hash" ]; then
    echo "Error: sha256 mismatch for '$pkg' (manifest vs file)." >&2
    exit 1
  fi

  manifest_profile="$(jq -r --arg pkg "$pkg" '.artifacts[] | select(.package == $pkg) | .profile' "$MANIFEST")"
  build_profile="$(jq -r '.build_profile' "$MANIFEST")"
  if [ "$manifest_profile" != "$build_profile" ]; then
    echo "Error: profile mismatch for '$pkg'." >&2
    exit 1
  fi
done

workspace_version="$(awk -F'"' '/^version = "/ {print $2; exit}' "$REPO_ROOT/Cargo.toml")"
manifest_version="$(jq -r '.version' "$MANIFEST")"
if [ "$manifest_version" != "$workspace_version" ]; then
  echo "Error: manifest version '$manifest_version' != workspace '$workspace_version'." >&2
  exit 1
fi

echo "Verified dist manifest: $artifact_count artifacts, commit $(jq -r '.commit' "$MANIFEST" | cut -c1-8), profile $(jq -r '.build_profile' "$MANIFEST")."
