#!/usr/bin/env sh
set -eu

SOROBAN_RPC_URL="${SOROBAN_RPC_URL:-${1:-https://soroban-testnet.stellar.org}}"
TIMEOUT="${RPC_TIMEOUT:-10}"

printf "Probing Soroban RPC endpoint: %s\n" "$SOROBAN_RPC_URL"

# 1. Probe getHealth
HEALTH_REQ='{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
HEALTH_RESP="$(curl -s -f -m "$TIMEOUT" -X POST \
  -H "Content-Type: application/json" \
  -d "$HEALTH_REQ" \
  "$SOROBAN_RPC_URL" 2>/dev/null || true)"

if [ -z "$HEALTH_RESP" ]; then
  echo "Error: Failed to connect to Soroban RPC endpoint or request timed out." >&2
  exit 1
fi

# Check if JSONRPC error occurred
if echo "$HEALTH_RESP" | grep -q '"error"'; then
  echo "Error: getHealth returned an RPC error:" >&2
  echo "$HEALTH_RESP" >&2
  exit 1
fi

# Check for health status
if ! echo "$HEALTH_RESP" | grep -q '"status"[[:space:]]*:[[:space:]]*"healthy"'; then
  echo "Error: Soroban RPC reported unhealthy status:" >&2
  echo "$HEALTH_RESP" >&2
  exit 1
fi

# 2. Probe getLatestLedger
LEDGER_REQ='{"jsonrpc":"2.0","id":2,"method":"getLatestLedger"}'
LEDGER_RESP="$(curl -s -f -m "$TIMEOUT" -X POST \
  -H "Content-Type: application/json" \
  -d "$LEDGER_REQ" \
  "$SOROBAN_RPC_URL" 2>/dev/null || true)"

if [ -z "$LEDGER_RESP" ]; then
  echo "Error: getLatestLedger probe failed or timed out." >&2
  exit 1
fi

if echo "$LEDGER_RESP" | grep -q '"error"'; then
  echo "Error: getLatestLedger returned an RPC error:" >&2
  echo "$LEDGER_RESP" >&2
  exit 1
fi

if ! echo "$LEDGER_RESP" | grep -q '"sequence"'; then
  echo "Error: getLatestLedger response missing sequence field." >&2
  exit 1
fi

LEDGER_SEQ="$(echo "$LEDGER_RESP" | sed -n 's/.*"sequence"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p')"

printf "RPC Status: healthy\n"
printf "Latest Ledger Sequence: %s\n" "$LEDGER_SEQ"
printf "Health probe succeeded for %s\n" "$SOROBAN_RPC_URL"
exit 0
