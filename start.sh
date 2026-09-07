#!/usr/bin/env bash
set -euo pipefail

print_banner() {
  [ -n "${PENCARIMOVIE_NO_BANNER:-}" ] && return 0
  local orange="" reset=""
  if [ -t 1 ]; then
    orange="$(printf '\033[38;5;208m')"
    reset="$(printf '\033[0m')"
  fi
  printf '%s' "$orange"
  cat <<'EOF'

 ========================================
          PencariMovie Server
 ========================================

EOF
  printf '%s' "$reset"
}

print_banner

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRANKENPHP_BIN="$ROOT_DIR/bin/frankenphp"
HOST="0.0.0.0"
PORT="8088"

# Detect LAN IP via default route (avoids virtual adapter IPs like Docker/VPN)
LAN_IP=""

# WSL: use powershell.exe to get Windows host's real LAN IP via route print
if grep -qi microsoft /proc/version 2>/dev/null && command -v powershell.exe >/dev/null 2>&1; then
  LAN_IP=$(powershell.exe -Command "route print -4 0.0.0.0 | Select-String '0.0.0.0\s+0.0.0.0' | ForEach-Object { (\$_ -split '\s+')[4] }" 2>/dev/null | tr -d '\r' | head -1)
fi

# macOS: use ipconfig getifaddr en0 or en1
if [ "$(uname -s 2>/dev/null)" = "Darwin" ]; then
  LAN_IP=$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || true)
fi

# Standard Linux / Libwrt: use ip route get (avoids listing all adapters)
if [ -z "$LAN_IP" ] && command -v ip >/dev/null 2>&1; then
  LAN_IP=$(ip route get 8.8.8.8 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") {print $(i+1); exit}}')
fi

# Fallback: hostname -I
if [ -z "$LAN_IP" ] && command -v hostname >/dev/null 2>&1; then
  LAN_IP=$(hostname -I 2>/dev/null | awk '{print $1}')
fi

# Fallback: ifconfig (BusyBox / Libwrt / OpenWrt)
if [ -z "$LAN_IP" ] && command -v ifconfig >/dev/null 2>&1; then
  LAN_IP=$(ifconfig 2>/dev/null | awk '
    /inet / {
      for (i=1; i<=NF; i++) {
        if ($i ~ /^addr:/) { sub(/^addr:/, "", $i); ip=$i }
        else if ($i == "inet" && $(i+1) !~ /^127\./) { ip=$(i+1) }
      }
      if (ip != "" && ip !~ /^127\./) { print ip; exit }
    }
  ')
fi

print_urls() {
  echo ""
  echo "  Local:    http://127.0.0.1:$PORT"
  if [ -n "$LAN_IP" ]; then
    echo "  Network:  http://$LAN_IP:$PORT"
  fi
  echo "  CLI:      pms [start|stop|restart|uninstall]"
  echo "  Stop:     pms stop"
  echo "  Restart:  pms restart"
  echo "  Uninstall: pms uninstall"
  echo ""
}

# If the server is already listening on the port, do not start a second instance.
if command -v curl >/dev/null 2>&1 && curl -s -m 2 "http://127.0.0.1:$PORT/" >/dev/null 2>&1; then
  echo "Server is already running on port $PORT."
  echo "  Local:    http://127.0.0.1:$PORT"
  if [ -n "$LAN_IP" ]; then
    echo "  Network:  http://$LAN_IP:$PORT"
  fi
  echo "  CLI:      pms [start|stop|restart|uninstall]"
  echo "  Stop:     pms stop"
  echo "  Restart:  pms restart"
  echo "  Uninstall: pms uninstall"
  exit 0
fi

# Ensure the IPC worker wrapper and runtime are executable. Windows-created
# tarballs often lose the +x bit, which makes ProcessRunner fail with
# "Permission denied" when it spawns bin/php.
chmod u+x "$FRANKENPHP_BIN" "$ROOT_DIR/bin/php" "$ROOT_DIR/bin/addon" 2>/dev/null || true

ADDON_LOG="$ROOT_DIR/addon.log"
ADDON_PID_FILE="$ROOT_DIR/.addon.pid"

if ! command -v curl >/dev/null 2>&1 || ! curl -s -m 1 http://127.0.0.1:8089/ >/dev/null 2>&1; then
  if [ -x "$ROOT_DIR/bin/addon" ]; then
    echo "Starting Addon service on port 8089..."
    nohup "$ROOT_DIR/bin/addon" >"$ADDON_LOG" 2>&1 &
    echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
  elif [ -f "$ROOT_DIR/addon.js" ]; then
    if command -v bun >/dev/null 2>&1; then
      echo "Starting Addon service with bun on port 8089..."
      nohup bun "$ROOT_DIR/addon.js" >"$ADDON_LOG" 2>&1 &
      echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
    elif command -v node >/dev/null 2>&1; then
      echo "Starting Addon service with node on port 8089..."
      nohup node "$ROOT_DIR/addon.js" >"$ADDON_LOG" 2>&1 &
      echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
    fi
  fi
fi

if [ -x "$FRANKENPHP_BIN" ]; then
  echo "Starting PencariMovie Server with FrankenPHP..."
  export PATH="$ROOT_DIR/bin:$PATH"
  export PHP_BINDIR="$ROOT_DIR/bin"
  export PHPRC="$ROOT_DIR/bin"
  nohup "$FRANKENPHP_BIN" php-server --listen "$HOST:$PORT" --root "$ROOT_DIR" >/dev/null 2>&1 &
  echo $! > "$ROOT_DIR/.frankenphp.pid"
  print_urls
  echo "FrankenPHP server started (PID $(cat "$ROOT_DIR/.frankenphp.pid"))."
  echo "Warming up IPC workers..."
  if [ -x "$ROOT_DIR/bin/php" ]; then
    "$ROOT_DIR/bin/php" "$ROOT_DIR/warmup-ipc.php" >/dev/null 2>&1 || true
  else
    php "$ROOT_DIR/warmup-ipc.php" >/dev/null 2>&1 || true
  fi
  exit 0
fi

if ! command -v php >/dev/null 2>&1; then
  echo "PHP or FrankenPHP is required but was not found."
  echo "Place FrankenPHP at $FRANKENPHP_BIN or install PHP in PATH."
  exit 1
fi
echo "Starting PencariMovie Server with PHP..."
nohup php -S "$HOST:$PORT" "$ROOT_DIR/router.php" >/dev/null 2>&1 &
echo $! > "$ROOT_DIR/.php-server.pid"
print_urls
echo "PHP server started (PID $(cat "$ROOT_DIR/.php-server.pid"))."
echo "Warming up IPC workers..."
php "$ROOT_DIR/warmup-ipc.php" >/dev/null 2>&1 || true
