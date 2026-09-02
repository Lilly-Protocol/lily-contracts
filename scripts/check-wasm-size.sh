#!/bin/sh
# Wasm size regression gate (#125).
#
# Compares the release wasm size of each contract package against the
# committed baseline (ci/wasm-size-baseline.json) and fails when a size
# regresses beyond the committed tolerance (default 5%).
#
# Exit codes:
#   0 - all sizes within budget
#   1 - regression beyond tolerance, or a package is missing from the baseline
#
# Re-baselining after an intended growth:
#   make build-wasm && sh scripts/record-wasm-size-baseline.sh

set -eu

BASELINE="${1:-ci/wasm-size-baseline.json}"
WASM_DIR="${WASM_DIR:-target/wasm32v1-none/release}"

if [ ! -f "$BASELINE" ]; then
    echo "wasm-size: FILL (missing baseline: $BASELINE)" >&2
    exit 1
fi

tolerance=$(sed -n 's/.*"regression_tolerance"[[:space:]]*:[[:space:]]*\([0-9.]*\).*/\1/p' "$BASELINE")
if [ -z "$tolerance" ]; then
    tolerance=0.05
fi

# awk does the float math (POSIX, no bc dependency).
check() {
    pkg="$1"
    file="$WASM_DIR/$1.wasm"
    if [ ! -f "$file" ]; then
        echo "wasm-size: MISSING $pkg ($file not found)" >&2
        return 1
    fi
    actual=$(wc -c < "$file")
    base=$(sed -n "s/.*\"$pkg\"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p" "$BASELINE")
    if [ -z "$base" ]; then
        echo "wasm-size: UNTRACKED $pkg ($actual B - add it to $BASELINE)" >&2
        return 1
    fi
    verdict=$(awk -v a="$actual" -v b="$base" -v t="$tolerance" \
        'BEGIN { if (b > 0 && a > b * (1 + t)) print "REGRESSION"; else print "OK" }')
    if [ "$verdict" = "REGRESSION" ]; then
        echo "wasm-size: FAIL $pkg  actual=$actual B  baseline=$base B  tolerance=${tolerance}" >&2
        return 1
    fi
    echo "wasm-size: $pkg ${actual} B (baseline ${base} B, tolerance ${tolerance})"
    return 0
}

echo "wasm-size: checking against $BASELINE (tolerance ${tolerance})"

status=0
for pkg in protocol identity wallet payments; do
    check "$pkg" || status=1
done

if [ "$status" -eq 0 ]; then
    echo "wasm-size: PASS"
else
    echo "wasm-size: FAIL (update the baseline intentionally or shrink the artifacts)" >&2
fi
exit "$status"
