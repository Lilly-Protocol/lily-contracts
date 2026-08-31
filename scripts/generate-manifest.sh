#!/usr/bin/env sh
set -eu

# Generate a manifest for the compiled Wasm artifacts.
#
# Produces dist/manifest.json containing per-artifact metadata:
#   - sha256 hash
#   - package version from Cargo.toml
#   - git commit hash
#   - build profile
#
# Usage:
#   ./scripts/generate-manifest.sh [output-file]

OUTPUT_FILE="${1:-dist/manifest.json}"
ARTIFACTS_DIR="dist"
WASM_DIR="target/wasm32v1-none/release"
CONTRACT_PACKAGES="protocol identity wallet payments"
PROFILE="release"

mkdir -p "$ARTIFACTS_DIR"

if command -v git >/dev/null 2>&1 && [ -d .git ]; then
  git_commit=$(git rev-parse HEAD)
else
  git_commit="unknown"
fi

{
  printf '{'
  printf '"generated_at":"%s",' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '"git_commit":"%s",' "$git_commit"
  printf '"profile":"%s",' "$PROFILE"
  printf '"artifacts":['

  first=1
  for pkg in $CONTRACT_PACKAGES; do
    wasm_path="$ARTIFACTS_DIR/$pkg.wasm"

    if [ ! -f "$wasm_path" ]; then
      echo "Error: $wasm_path not found. Run 'make artifacts' first." >&2
      exit 1
    fi

    version=$(grep -m 1 '^version' "contracts/$pkg/Cargo.toml" | sed 's/.*"\([^"]*\)".*/\1/')
    hash=$(sha256sum "$wasm_path" | awk '{print $1}')

    if [ "$first" -eq 1 ]; then
      first=0
    else
      printf ','
    fi

    printf '{"package":"%s","version":"%s","file":"%s","sha256":"%s","profile":"%s"}' "$pkg" "$version" "$wasm_path" "$hash" "$PROFILE"
  done

  printf ']}'
  printf '\n'
} > "$OUTPUT_FILE"

echo "Manifest written to $OUTPUT_FILE"
