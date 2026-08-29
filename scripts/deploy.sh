#!/usr/bin/env sh
set -eu

NETWORK="${STELLAR_NETWORK:-testnet}"
SOURCE_ACCOUNT="${STELLAR_SOURCE_ACCOUNT:-alice}"
WASM_DIR="${WASM_DIR:-dist}"
OUTPUT_FILE="${OUTPUT_FILE:-deployed-contracts.json}"
DRY_RUN="${DRY_RUN:-false}"

# Parse optional command line flags
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)
      DRY_RUN="true"
      shift
      ;;
    --network=*)
      NETWORK="${1#*=}"
      shift
      ;;
    --source=*)
      SOURCE_ACCOUNT="${1#*=}"
      shift
      ;;
    --output=*)
      OUTPUT_FILE="${1#*=}"
      shift
      ;;
    *)
      printf "Unknown flag: %s\n" "$1" >&2
      shift
      ;;
  esac
done

printf "=== Lilly Contracts Deployment Pipeline ===\n"
printf "Target Network  : %s\n" "$NETWORK"
printf "Source Account  : %s\n" "$SOURCE_ACCOUNT"
printf "Wasm Directory  : %s\n" "$WASM_DIR"
printf "Output File     : %s\n" "$OUTPUT_FILE"
printf "Dry Run Mode    : %s\n\n" "$DRY_RUN"

# Required contracts in topological dependency order
CONTRACT_ORDER="protocol identity wallet payments"

# Verify all wasm binaries exist if not in dry-run mode
if [ "$DRY_RUN" != "true" ]; then
  for contract in $CONTRACT_ORDER; do
    WASM_PATH="$WASM_DIR/$contract.wasm"
    if [ ! -f "$WASM_PATH" ]; then
      printf "Error: Wasm artifact '%s' not found. Run 'make artifacts' before deploying.\n" "$WASM_PATH" >&2
      exit 1
    fi
  done
fi

PROTOCOL_ID=""
IDENTITY_ID=""
WALLET_ID=""
PAYMENTS_ID=""

for contract in $CONTRACT_ORDER; do
  printf "Deploying contract: %s ...\n" "$contract"
  WASM_PATH="$WASM_DIR/$contract.wasm"
  
  if [ "$DRY_RUN" = "true" ]; then
    DEPLOYED_ID="DRY_RUN_ID_$(echo "$contract" | tr '[:lower:]' '[:upper:]')"
    printf "  [DRY-RUN] stellar contract deploy --wasm %s --source %s --network %s\n" "$WASM_PATH" "$SOURCE_ACCOUNT" "$NETWORK"
    printf "  [DRY-RUN] Simulated ID: %s\n" "$DEPLOYED_ID"
  else
    DEPLOYED_ID=$(stellar contract deploy --wasm "$WASM_PATH" --source "$SOURCE_ACCOUNT" --network "$NETWORK")
    printf "  Deployed ID: %s\n" "$DEPLOYED_ID"
  fi

  case "$contract" in
    protocol) PROTOCOL_ID="$DEPLOYED_ID" ;;
    identity) IDENTITY_ID="$DEPLOYED_ID" ;;
    wallet)   WALLET_ID="$DEPLOYED_ID" ;;
    payments) PAYMENTS_ID="$DEPLOYED_ID" ;;
  esac
done

# Write structured JSON outputs
printf "Saving deployment manifest to %s ...\n" "$OUTPUT_FILE"

cat <<EOF > "$OUTPUT_FILE"
{
  "network": "$NETWORK",
  "sourceAccount": "$SOURCE_ACCOUNT",
  "deployedAt": "$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)",
  "contracts": {
    "protocol": "$PROTOCOL_ID",
    "identity": "$IDENTITY_ID",
    "wallet": "$WALLET_ID",
    "payments": "$PAYMENTS_ID"
  }
}
EOF

printf "✓ Deployment completed successfully.\n"
