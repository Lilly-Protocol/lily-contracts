#!/usr/bin/env sh
set -eu

# Deploy all Lily Protocol contracts to a target network.
#
# Usage:
#   ./scripts/deploy.sh <network> <source-account> [output-file]
#
# Examples:
#   ./scripts/deploy.sh testnet alice
#   ./scripts/deploy.sh local alice ./config/deployed-contracts.json
#
# The script records returned contract IDs to the output file (default:
# ./config/deployed-contracts.json) so that init-contracts.sh can consume them.

NETWORK="${1:-}"
SOURCE="${2:-}"
OUTPUT_FILE="${3:-./config/deployed-contracts.json}"
DRY_RUN="${DRY_RUN:-false}"

CONTRACT_PACKAGES="protocol identity wallet payments"
WASM_DIR="target/wasm32v1-none/release"

if [ -z "$NETWORK" ] || [ -z "$SOURCE" ]; then
  echo "Usage: $0 <network> <source-account> [output-file]" >&2
  exit 1
fi

if [ "$DRY_RUN" != "false" ]; then
  echo "Dry run mode enabled. No contracts will be deployed."
fi

if ! command -v stellar >/dev/null 2>&1; then
  echo "Error: stellar CLI is not installed." >&2
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_FILE")"

# Start with an empty JSON object.
{
  printf '{'
  first=1
  for pkg in $CONTRACT_PACKAGES; do
    wasm_path="$WASM_DIR/$pkg.wasm"

    if [ ! -f "$wasm_path" ]; then
      echo "Error: $wasm_path not found. Run 'make build-wasm' first." >&2
      exit 1
    fi

    if [ "$DRY_RUN" != "false" ]; then
      contract_id="CDRYRUN${pkg}000000000000000000000000000"
      echo "Dry run: would deploy $pkg -> $contract_id"
    else
      echo "Deploying $pkg..."
      contract_id=$(stellar contract deploy \
        --wasm "$wasm_path" \
        --source-account "$SOURCE" \
        --network "$NETWORK" \
        2>&1 | tail -n 1 | tr -d '[:space:]')

      if [ -z "$contract_id" ]; then
        echo "Error: failed to deploy $pkg (no contract id returned)." >&2
        exit 1
      fi
    fi

    if [ "$first" -eq 1 ]; then
      first=0
    else
      printf ','
    fi
    printf '"%s":"%s"' "$pkg" "$contract_id"
  done
  printf '}\n'
} > "$OUTPUT_FILE"

echo "Deployment complete. Contract IDs written to $OUTPUT_FILE"
if command -v jq >/dev/null 2>&1; then
  jq . "$OUTPUT_FILE"
else
  cat "$OUTPUT_FILE"
fi

if [ "$DRY_RUN" = "false" ]; then
  echo ""
  echo "Next step: initialize contracts with ./scripts/init-contracts.sh"
fi
