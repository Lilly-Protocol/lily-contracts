#!/usr/bin/env sh
set -eu

ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"
OUTPUT_FILE="${ARTIFACTS_DIR}/manifest.json"

if [ ! -d "$ARTIFACTS_DIR" ]; then
  echo "Error: artifacts directory '$ARTIFACTS_DIR' does not exist." >&2
  exit 1
fi

GIT_COMMIT="$(git rev-parse HEAD 2>/dev/null || echo "unknown")"
GIT_BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")"
BUILD_PROFILE="${BUILD_PROFILE:-release}"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

# Extract workspace version from root Cargo.toml if available
VERSION="$(grep -m1 '^version = ' Cargo.toml 2>/dev/null | cut -d '"' -f2 || echo "0.1.0")"

mkdir -p "$ARTIFACTS_DIR"

first=1
artifacts_json=""

for wasm_file in "$ARTIFACTS_DIR"/*.wasm; do
  [ -e "$wasm_file" ] || continue
  filename="$(basename "$wasm_file")"
  pkg_name="${filename%.wasm}"
  
  if command -v sha256sum >/dev/null 2>&1; then
    hash="$(sha256sum "$wasm_file" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    hash="$(shasum -a 256 "$wasm_file" | awk '{print $1}')"
  else
    echo "Error: neither sha256sum nor shasum is available." >&2
    exit 1
  fi
  
  size="$(wc -c < "$wasm_file" | tr -d ' ')"
  
  entry="    {
      \"package\": \"$pkg_name\",
      \"file\": \"$filename\",
      \"sha256\": \"$hash\",
      \"size_bytes\": $size,
      \"version\": \"$VERSION\",
      \"profile\": \"$BUILD_PROFILE\"
    }"

  if [ "$first" -eq 1 ]; then
    artifacts_json="$entry"
    first=0
  else
    artifacts_json="$artifacts_json,
$entry"
  fi
done

cat <<EOF > "$OUTPUT_FILE"
{
  "commit": "$GIT_COMMIT",
  "branch": "$GIT_BRANCH",
  "build_profile": "$BUILD_PROFILE",
  "generated_at": "$TIMESTAMP",
  "version": "$VERSION",
  "artifacts": [
$artifacts_json
  ]
}
EOF

printf "Generated contract artifact manifest at %s\n" "$OUTPUT_FILE"
