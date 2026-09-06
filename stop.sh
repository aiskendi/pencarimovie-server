#!/usr/bin/env bash
set -euo pipefail

HOST="0.0.0.0"
PORT="8088"
ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Stopping PencariMovie Server on $HOST:$PORT..."

TUNNEL_PID_FILE="$ROOT_DIR/storage/tunnel/cloudflared.pid"
TUNNEL_CONFIG="$ROOT_DIR/storage/tunnel/config.yml"
if [ -f "$TUNNEL_PID_FILE" ]; then
  TUNNEL_PID="$(cat "$TUNNEL_PID_FILE" 2>/dev/null || true)"
  if [ -n "${TUNNEL_PID:-}" ] && kill -0 "$TUNNEL_PID" 2>/dev/null; then
    echo "Stopping Cloudflare tunnel PID $TUNNEL_PID"
    kill "$TUNNEL_PID" 2>/dev/null || true
    sleep 0.2
    kill -9 "$TUNNEL_PID" 2>/dev/null || true
  fi
  rm -f "$TUNNEL_PID_FILE" 2>/dev/null || true
fi
if command -v pkill >/dev/null 2>&1 && [ -n "$TUNNEL_CONFIG" ]; then
  pkill -f "$TUNNEL_CONFIG" 2>/dev/null || true
fi
rm -f "$ROOT_DIR/storage/tunnel/state.json" 2>/dev/null || true

if [ -f "$ROOT_DIR/.addon.pid" ]; then
  ADDON_PID="$(cat "$ROOT_DIR/.addon.pid" 2>/dev/null || true)"
  if [ -n "${ADDON_PID:-}" ] && kill -0 "$ADDON_PID" 2>/dev/null; then
    kill "$ADDON_PID" 2>/dev/null || true
    kill -9 "$ADDON_PID" 2>/dev/null || true
  fi
  rm -f "$ROOT_DIR/.addon.pid" 2>/dev/null || true
fi

pkill -9 -f "addon\.js" 2>/dev/null || true
pkill -9 -f "bin/addon" 2>/dev/null || true
pkill -9 -f "node.*addon" 2>/dev/null || true
pkill -9 -f "bun.*addon" 2>/dev/null || true

if command -v lsof >/dev/null 2>&1; then
  ADDON_PIDS="$(lsof -ti tcp:8089 -sTCP:LISTEN 2>/dev/null || true)"
  [ -n "$ADDON_PIDS" ] && kill -9 $ADDON_PIDS 2>/dev/null || true
fi

if command -v fuser >/dev/null 2>&1; then
  fuser -k -9 8089/tcp 2>/dev/null || true
fi

for PID_FILE in "$ROOT_DIR/.frankenphp.pid" "$ROOT_DIR/.php-server.pid"; do
  if [ -f "$PID_FILE" ]; then
    PID="$(cat "$PID_FILE" 2>/dev/null || true)"
    if [ -n "${PID:-}" ] && kill -0 "$PID" 2>/dev/null; then
      echo "Killing process from $(basename "$PID_FILE") PID $PID"
      kill "$PID" 2>/dev/null || true
    fi
    rm -f "$PID_FILE" 2>/dev/null || true
  fi
done

if command -v lsof >/dev/null 2>&1; then
  PIDS="$(lsof -ti tcp:"$PORT" -sTCP:LISTEN || true)"
elif command -v fuser >/dev/null 2>&1; then
  PIDS="$(fuser "$PORT"/tcp 2>/dev/null || true)"
else
  echo "Install lsof or fuser to stop by port automatically."
  exit 1
fi

if [ -z "${PIDS:-}" ]; then
  echo "No process is listening on $HOST:$PORT."
  exit 0
fi

echo "$PIDS" | tr ' ' '\n' | while read -r PID; do
  [ -z "$PID" ] && continue
  echo "Killing process PID $PID"
  kill "$PID" 2>/dev/null || true
done

echo "Stop command completed."
