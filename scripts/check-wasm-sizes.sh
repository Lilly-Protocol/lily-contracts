#!/usr/bin/env sh
set -eu

BASELINE_FILE="${BASELINE_FILE:-wasm-size-baseline.json}"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"
WASM_DIR="${WASM_DIR:-target/wasm32v1-none/release}"

if [ ! -f "$BASELINE_FILE" ]; then
  echo "Error: baseline file '$BASELINE_FILE' not found." >&2
  exit 1
fi

packages="protocol identity wallet payments"
has_failure=0

printf "%-12s | %-12s | %-12s | %-12s | %-10s\n" "Contract" "Actual (B)" "Baseline (B)" "Max Limit" "Status"
printf -- "-------------+--------------+--------------+--------------+-----------\n"

for pkg in $packages; do
  wasm_path=""
  if [ -f "${ARTIFACTS_DIR}/${pkg}.wasm" ]; then
    wasm_path="${ARTIFACTS_DIR}/${pkg}.wasm"
  elif [ -f "${WASM_DIR}/${pkg}.wasm" ]; then
    wasm_path="${WASM_DIR}/${pkg}.wasm"
  fi

  if [ -z "$wasm_path" ] || [ ! -f "$wasm_path" ]; then
    printf "%-12s | %-12s | %-12s | %-12s | %-10s\n" "$pkg" "NOT FOUND" "-" "-" "FAIL"
    has_failure=1
    continue
  fi

  actual_size=$(wc -c < "$wasm_path" | tr -d ' ')
  
  # Extract baseline and max limit from json
  max_limit=$(grep -A 3 "\"$pkg\":" "$BASELINE_FILE" | grep '"max_limit_bytes":' | tr -dc '0-9')
  baseline=$(grep -A 3 "\"$pkg\":" "$BASELINE_FILE" | grep '"baseline_bytes":' | tr -dc '0-9')
  
  if [ -z "$max_limit" ]; then
    max_limit=131072
  fi
  if [ -z "$baseline" ]; then
    baseline=65536
  fi

  if [ "$actual_size" -gt "$max_limit" ]; then
    status="EXCEEDED"
    has_failure=1
  else
    status="OK"
  fi

  printf "%-12s | %-12s | %-12s | %-12s | %-10s\n" "$pkg" "$actual_size" "$baseline" "$max_limit" "$status"
done

printf -- "-------------+--------------+--------------+--------------+-----------\n"

if [ "$has_failure" -ne 0 ]; then
  echo "Error: Wasm size regression gate failed." >&2
  exit 1
fi

echo "All Wasm artifacts are within size thresholds."
