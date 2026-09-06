#!/usr/bin/env sh
# Local-only manifest pipeline test using rustc-compiled minimal Wasm modules.
# NOT used in CI — does not produce lily-contract bytecode.
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_TARGET="${WASM_TARGET:-wasm32v1-none}"
WASM_DIR="${WASM_DIR:-target/${WASM_TARGET}/release}"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist}"
PACKAGES="${PACKAGES:-protocol identity wallet payments}"

if ! command -v rustc >/dev/null 2>&1; then
  echo "Error: rustc required for offline manifest test." >&2
  exit 1
fi

TOOLCHAIN="${ARTIFACTS_SMOKE_TOOLCHAIN:-stable}"
export RUSTUP_TOOLCHAIN="$TOOLCHAIN"

if ! rustup target list --installed --toolchain "$TOOLCHAIN" 2>/dev/null | grep -qx "$WASM_TARGET"; then
  echo "Installing $WASM_TARGET for toolchain $TOOLCHAIN..."
  rustup target add "$WASM_TARGET" --toolchain "$TOOLCHAIN"
fi

tmp_src="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_src"
}
trap cleanup EXIT INT HUP

cat > "$tmp_src/minimal.rs" <<'EOF'
#![no_std]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
EOF

mkdir -p "$WASM_DIR" "$ARTIFACTS_DIR"
for pkg in $PACKAGES; do
  rustc \
    --crate-type cdylib \
    --target "$WASM_TARGET" \
    "$tmp_src/minimal.rs" \
    -o "$WASM_DIR/${pkg}.wasm"
  cp "$WASM_DIR/${pkg}.wasm" "$ARTIFACTS_DIR/${pkg}.wasm"
done

(
  cd "$REPO_ROOT"
  BUILD_PROFILE="${BUILD_PROFILE:-release}" ./scripts/generate-manifest.sh
)

"$REPO_ROOT/scripts/verify-dist-manifest.sh"
"$REPO_ROOT/scripts/assert-contract-artifacts-bundle.sh"

echo "Offline manifest pipeline test passed (minimal Wasm, not contract bytecode)."
