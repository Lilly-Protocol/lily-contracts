#!/usr/bin/env sh
# Assert dist/ contains every file CI uploads as contract-artifacts.
set -eu

ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"

required_files="
${ARTIFACTS_DIR}/protocol.wasm
${ARTIFACTS_DIR}/identity.wasm
${ARTIFACTS_DIR}/wallet.wasm
${ARTIFACTS_DIR}/payments.wasm
${ARTIFACTS_DIR}/manifest.json
"

count=0
for file in $required_files; do
  if [ ! -f "$file" ]; then
    echo "Error: missing contract-artifacts bundle file: $file" >&2
    exit 1
  fi
  count=$((count + 1))
done

echo "Contract-artifacts bundle complete (${count} files)."
