#!/usr/bin/env sh
set -eu

# Bootstrap a local Stellar standalone network for development.
#
# Starts the official stellar/quickstart image in standalone mode, waits for
# the Soroban RPC endpoint to become healthy, and prints teardown instructions.
#
# Usage:
#   ./scripts/bootstrap-local.sh [container-name] [rpc-port]
#
# Defaults:
#   container-name: lily-local
#   rpc-port: 8000

CONTAINER_NAME="${1:-lily-local}"
RPC_PORT="${2:-8000}"
IMAGE="stellar/quickstart:latest"
RPC_URL="http://localhost:${RPC_PORT}/soroban/rpc"

if ! command -v docker >/dev/null 2>&1; then
  echo "Error: docker is not installed." >&2
  exit 1
fi

if docker ps -a --format '{{.Names}}' | grep -qx "$CONTAINER_NAME"; then
  echo "Container '$CONTAINER_NAME' already exists."
  if docker ps --format '{{.Names}}' | grep -qx "$CONTAINER_NAME"; then
    echo "It is already running. RPC should be available at $RPC_URL"
    exit 0
  fi
  echo "Starting existing container..."
  docker start "$CONTAINER_NAME"
else
  echo "Creating and starting local Stellar standalone network..."
  echo "Container: $CONTAINER_NAME"
  echo "RPC URL:   $RPC_URL"
  docker run -d \
    --name "$CONTAINER_NAME" \
    -p "$RPC_PORT:8000" \
    --platform linux/amd64 \
    "$IMAGE" \
    --standalone \
    --enable-soroban-rpc
fi

echo "Waiting for Soroban RPC to become healthy..."
attempts=0
max_attempts=60
while [ "$attempts" -lt "$max_attempts" ]; do
  attempts=$((attempts + 1))
  if curl -sf -X POST "$RPC_URL" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"getHealth","params":{}}' >/dev/null 2>&1; then
    break
  fi
  printf '.'
  sleep 2
done
printf '\n'

if [ "$attempts" -eq "$max_attempts" ]; then
  echo "Error: RPC did not become healthy within $((max_attempts * 2)) seconds." >&2
  echo "Check logs with: docker logs $CONTAINER_NAME" >&2
  exit 1
fi

echo "Local Stellar standalone network is ready."
echo "RPC URL: $RPC_URL"
echo ""
echo "To use this network with the init script, set the network to 'local' or pass the RPC URL directly."
echo ""
echo "Teardown commands:"
echo "  docker stop $CONTAINER_NAME"
echo "  docker rm $CONTAINER_NAME"
