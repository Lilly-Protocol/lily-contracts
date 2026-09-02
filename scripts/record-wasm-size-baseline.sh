#!/bin/sh
# Records the current release wasm sizes into ci/wasm-size-baseline.json.
# Run after an intended, reviewed growth of the artifacts:
#   make build-wasm && sh scripts/record-wasm-size-baseline.sh
set -eu

WASM_DIR="${WASM_DIR:-target/wasm32v1-none/release}"
OUT="ci/wasm-size-baseline.json"
pkgs="protocol identity wallet payments"
total=0
for pkg in $pkgs; do
    if [ ! -f "$WASM_DIR/$pkg.wasm" ]; then
        echo "record-wasm-baseline: missing $WASM_DIR/$pkg.wasm (run: make build-wasm)" >&2
        exit 1
    fi
    total=$((total + 1))
done

mkdir -p ci
{
    printf '{\n'
    printf '  "updated": "%s",\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf '  "regression_tolerance": 0.05,\n'
    printf '  "max_bytes": {\n'
    i=0
    for pkg in $pkgs; do
        i=$((i + 1))
        size=$(wc -c < "$WASM_DIR/$pkg.wasm")
        case $size in (*[!0-9]* | '' | ' '*) size=$(printf '%s' "$size" | tr -d '[:space:]') ;; esac
        if [ "$i" -lt "$total" ]; then
            printf '    "%s": %s,\n' "$pkg" "$size"
        else
            printf '    "%s": %s\n' "$pkg" "$size"
        fi
    done
    printf '  }\n'
    printf '}\n'
} > "$OUT"
echo "record-wasm-baseline: wrote $OUT"
cat "$OUT"
