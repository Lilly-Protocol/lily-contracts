#!/usr/bin/env sh
set -eu

RPC_URL="${SOROBAN_RPC_URL:-http://localhost:8000/soroban/rpc}"

printf "Probing Soroban RPC endpoint at: %s\n" "$RPC_URL"

# 1. Probe getHealth
HEALTH_RESP=$(curl -s -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' "$RPC_URL" 2>/dev/null || true)

if [ -z "$HEALTH_RESP" ]; then
  printf "Error: Unable to connect to Soroban RPC endpoint at %s\n" "$RPC_URL" >&2
  exit 1
fi

if ! echo "$HEALTH_RESP" | grep -q '"status":"healthy"'; then
  printf "Error: Soroban RPC health status is not healthy. Response: %s\n" "$HEALTH_RESP" >&2
  exit 1
fi

printf "✓ Soroban RPC health check passed (status: healthy)\n"

# 2. Probe getLatestLedger
LEDGER_RESP=$(curl -s -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"getLatestLedger"}' "$RPC_URL" 2>/dev/null || true)

if [ -z "$LEDGER_RESP" ]; then
  printf "Error: Failed to query latest ledger from %s\n" "$RPC_URL" >&2
  exit 1
fi

if ! echo "$LEDGER_RESP" | grep -q '"sequence"'; then
  printf "Error: getLatestLedger response missing ledger sequence. Response: %s\n" "$LEDGER_RESP" >&2
  exit 1
fi

LEDGER_SEQ=$(echo "$LEDGER_RESP" | grep -o '"sequence":[0-9]*' | cut -d: -f2 || true)
printf "✓ Latest ledger confirmed (sequence: %s)\n" "$LEDGER_SEQ"
printf "✓ Soroban RPC probe completed successfully.\n"
exit 0
