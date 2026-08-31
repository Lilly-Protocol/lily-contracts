#!/usr/bin/env sh
set -eu

# Soroban RPC health probe.
# Exits non-zero if the configured RPC endpoint is not healthy or has no ledger.
# Usage:
#   SOROBAN_RPC_URL=https://soroban-testnet.stellar.org ./scripts/rpc-health.sh

SOROBAN_RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"

if [ -z "$SOROBAN_RPC_URL" ]; then
  echo "Error: SOROBAN_RPC_URL is not set." >&2
  exit 1
fi

probe() {
  method="$1"
  curl -sf -X POST "$SOROBAN_RPC_URL" \
    -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":{}}"
}

health_raw=$(probe getHealth)
status=$(printf '%s\n' "$health_raw" | sed -n 's/.*"status":"\([^"]*\)".*/\1/p')

if [ "$status" != "healthy" ]; then
  echo "RPC health check failed: status=$status" >&2
  echo "Full response: $health_raw" >&2
  exit 1
fi

ledger_raw=$(probe getLatestLedger)
sequence=$(printf '%s\n' "$ledger_raw" | sed -n 's/.*"sequence":\([0-9]*\).*/\1/p')

if [ -z "$sequence" ] || [ "$sequence" -le 0 ]; then
  echo "RPC latest ledger check failed: sequence=$sequence" >&2
  echo "Full response: $ledger_raw" >&2
  exit 1
fi

echo "RPC is healthy."
echo "Latest ledger sequence: $sequence"
