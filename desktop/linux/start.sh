#!/bin/sh
# PencariMovie Server - console launcher (Linux).
#
#   ./start.sh                 foreground, log in this terminal
#   ./start.sh --open          also open the dashboard in a browser
#   PORT=9000 ./start.sh       different port
#
# For a tray icon use ./start-tray.sh; for login start use ./autostart-on.sh.
set -eu
cd "$(dirname "$0")"

PORT="${PORT:-8088}"
BIN="./pencarimovie-server"
[ -x "$BIN" ] || { echo "pencarimovie-server not found next to this script" >&2; exit 1; }

# Telegram part pool + streaming sockets need more than the default 1024 fds.
ulimit -n 65536 2>/dev/null || true

OPEN=0
for a in "$@"; do
  case "$a" in
    --open) OPEN=1 ;;
  esac
done

if [ "$OPEN" = "1" ]; then
  ( sleep 2
    URL="http://127.0.0.1:$PORT/"
    if command -v xdg-open >/dev/null 2>&1; then xdg-open "$URL" >/dev/null 2>&1 &
    elif command -v notify-send >/dev/null 2>&1; then notify-send "PencariMovie Server" "$URL"
    fi
  ) &
fi

# The engine prints its own banner with the storage path, worker count and rate
# limits, so this four-line one is only useful when a human is watching the
# terminal. PENCARIMOVIE_NO_BANNER=1 (set by the installers when they start the
# server detached) suppresses it instead of printing both.
if [ -z "${PENCARIMOVIE_NO_BANNER:-}" ]; then
  echo "PencariMovie Server"
  echo "  dashboard : http://127.0.0.1:$PORT/"
  echo "  add-on    : http://127.0.0.1:$PORT/manifest.json"
  echo "  Ctrl+C to stop"
fi
exec "$BIN" --port "$PORT" "$@"
