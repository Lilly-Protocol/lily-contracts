#!/usr/bin/env sh
set -eu

# Initialize all Lily Protocol contracts from a single config file.
#
# The config file (default: ./config/deploy.json) must contain:
#   {
#     "network": "testnet",
#     "source_account": "alice",
#     "admin": "G...",
#     "treasury": "G...",
#     "fee_bps": 100,
#     "contracts": {
#       "protocol": "C...",
#       "payments": "C...",
#       "identity": "C...",
#       "wallet": "C..."
#     }
#   }
#
# Usage:
#   ./scripts/init-contracts.sh [config-file]

CONFIG_FILE="${1:-./config/deploy.json}"

if ! command -v stellar >/dev/null 2>&1; then
  echo "Error: stellar CLI is not installed." >&2
  exit 1
fi

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: config file not found: $CONFIG_FILE" >&2
  exit 1
fi

# Minimal JSON extraction. Requires jq if available, otherwise falls back to sed.
json_get() {
  key="$1"
  file="$2"
  if command -v jq >/dev/null 2>&1; then
    jq -r "$key" "$file"
  else
    sed -n "s/.*\"$key\"\s*:\s*\"\([^\"]*\)\".*/\1/p" "$file"
  fi
}

NETWORK=$(json_get '.network' "$CONFIG_FILE")
SOURCE=$(json_get '.source_account' "$CONFIG_FILE")
ADMIN=$(json_get '.admin' "$CONFIG_FILE")
TREASURY=$(json_get '.treasury' "$CONFIG_FILE")
FEE_BPS=$(json_get '.fee_bps' "$CONFIG_FILE")
PROTOCOL_ID=$(json_get '.contracts.protocol' "$CONFIG_FILE")
PAYMENTS_ID=$(json_get '.contracts.payments' "$CONFIG_FILE")
IDENTITY_ID=$(json_get '.contracts.identity' "$CONFIG_FILE")
WALLET_ID=$(json_get '.contracts.wallet' "$CONFIG_FILE")

missing=""
for var in NETWORK SOURCE ADMIN TREASURY FEE_BPS PROTOCOL_ID PAYMENTS_ID IDENTITY_ID WALLET_ID; do
  eval "val=\$$var"
  if [ -z "$val" ] || [ "$val" = "null" ]; then
    missing="$missing $var"
  fi
done

if [ -n "$missing" ]; then
  echo "Error: missing required config fields:$missing" >&2
  exit 1
fi

validate_address() {
  name="$1"
  value="$2"
  case "$value" in
    G*|C*) ;;
    *)
      echo "Error: $name does not look like a Stellar address: $value" >&2
      exit 1
      ;;
  esac
}

validate_address "admin" "$ADMIN"
validate_address "treasury" "$TREASURY"
validate_address "protocol contract" "$PROTOCOL_ID"
validate_address "payments contract" "$PAYMENTS_ID"
validate_address "identity contract" "$IDENTITY_ID"
validate_address "wallet contract" "$WALLET_ID"

is_initialized() {
  contract_id="$1"
  stellar contract invoke \
    --id "$contract_id" \
    --source-account "$SOURCE" \
    --network "$NETWORK" \
    -- is_initialized 2>/dev/null | grep -q "true"
}

invoke_initialize() {
  contract_id="$1"
  contract_name="$2"
  shift 2
  echo "Initializing $contract_name ($contract_id)..."
  stellar contract invoke \
    --id "$contract_id" \
    --source-account "$SOURCE" \
    --network "$NETWORK" \
    -- initialize "$@"
}

# Protocol must be initialized first because it holds global config.
if is_initialized "$PROTOCOL_ID"; then
  echo "Protocol contract already initialized; skipping."
else
  invoke_initialize "$PROTOCOL_ID" "protocol" --admin "$ADMIN" --treasury "$TREASURY" --fee_bps "$FEE_BPS"
fi

# Identity and wallet only require an admin.
if is_initialized "$IDENTITY_ID"; then
  echo "Identity contract already initialized; skipping."
else
  invoke_initialize "$IDENTITY_ID" "identity" --admin "$ADMIN"
fi

if is_initialized "$WALLET_ID"; then
  echo "Wallet contract already initialized; skipping."
else
  invoke_initialize "$WALLET_ID" "wallet" --admin "$ADMIN"
fi

# Payments mirrors protocol configuration.
if is_initialized "$PAYMENTS_ID"; then
  echo "Payments contract already initialized; skipping."
else
  invoke_initialize "$PAYMENTS_ID" "payments" --admin "$ADMIN" --treasury "$TREASURY" --fee_bps "$FEE_BPS"
fi

echo "Initialization complete."
