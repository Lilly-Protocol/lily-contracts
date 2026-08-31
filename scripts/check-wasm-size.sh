#!/usr/bin/env sh
set -eu

# Compare compiled Wasm artifact sizes against the committed baseline.
# Exits non-zero if any contract grows beyond the configured threshold.
#
# Usage:
#   ./scripts/check-wasm-size.sh [baseline-file]

BASELINE_FILE="${1:-.wasm-size-baseline.json}"
TARGET_DIR="target/wasm32v1-none/release"
CONTRACTS="protocol identity wallet payments"

if [ ! -f "$BASELINE_FILE" ]; then
  echo "Error: baseline file not found: $BASELINE_FILE" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required to parse the baseline file." >&2
  exit 1
fi

threshold_percent=$(jq -r '.threshold_percent // 5.0' "$BASELINE_FILE")
threshold_bytes=$(jq -r '.threshold_bytes // 1024' "$BASELINE_FILE")

failed=0

for contract in $CONTRACTS; do
  wasm_path="$TARGET_DIR/$contract.wasm"
  baseline=$(jq -r ".contracts.$contract // 0" "$BASELINE_FILE")

  if [ "$baseline" -eq 0 ]; then
    echo "Warning: no baseline for $contract; skipping." >&2
    continue
  fi

  if [ ! -f "$wasm_path" ]; then
    echo "Error: $wasm_path not found. Run 'make build-wasm' first." >&2
    failed=1
    continue
  fi

  size=$(wc -c < "$wasm_path" | tr -d ' ')
  diff=$((size - baseline))
  abs_diff=${diff#-}

  # awk is used because sh lacks floating-point arithmetic.
  percent=$(awk -v d="$abs_diff" -v b="$baseline" 'BEGIN { printf "%.2f", (d / b) * 100 }')
  exceeds_percent=$(awk -v p="$percent" -v t="$threshold_percent" 'BEGIN { print (p > t) ? 1 : 0 }')
  exceeds_bytes=$(awk -v d="$abs_diff" -v t="$threshold_bytes" 'BEGIN { print (d > t) ? 1 : 0 }')

  if [ "$exceeds_percent" -eq 1 ] || [ "$exceeds_bytes" -eq 1 ]; then
    echo "FAIL: $contract.wasm size regression detected (baseline=$baseline, current=$size, +$abs_diff bytes, +$percent%)." >&2
    failed=1
  else
    echo "OK: $contract.wasm baseline=$baseline current=$size delta=$diff bytes ($percent%)."
  fi
done

if [ "$failed" -ne 0 ]; then
  echo "Wasm size regression check failed." >&2
  exit 1
fi

echo "Wasm size regression check passed."
