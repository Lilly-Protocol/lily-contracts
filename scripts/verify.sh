#!/usr/bin/env sh
# NIO-60 verification entry point — manifest acceptance (no cargo required).
set -eu

if [ -n "${NIO60_VERIFY_RUNNING:-}" ]; then
  echo "Error: verify.sh recursion detected" >&2
  exit 1
fi
NIO60_VERIFY_RUNNING=1
export NIO60_VERIFY_RUNNING

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"
make test-manifest
