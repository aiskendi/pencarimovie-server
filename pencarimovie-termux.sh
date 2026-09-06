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

# Fixed installation path under $HOME (like 9router) unless already running inside the project root
if [ -f "./backend.php" ] && [ -f "./start.sh" ]; then
  APP_DIR="."
  OLD_APP_DIR="pencarimovie-downloader"
else
  APP_DIR="${HOME:-/data/data/com.termux/files/home}/pencarimovie-server"
  OLD_APP_DIR="${HOME:-/data/data/com.termux/files/home}/pencarimovie-downloader"
fi
PORT="${PORT:-8088}"
HOST="${HOST:-0.0.0.0}"
REPO="aiskendi/pencarimovie-server"
FALLBACK_TAG="v1.0.0"

detect_target() {
  local arch os
  arch="$(uname -m)"
  os="$(uname -s)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)  echo "linux-x86_64" ;;
        aarch64|arm64) echo "linux-aarch64" ;;
        *) echo "Unsupported architecture: $arch"; exit 1 ;;
      esac
      ;;
    *) echo "Unsupported OS: $os. PencariMovie Server supports Linux, Android (Termux/APK), and Windows."; exit 1 ;;
  esac
}

usage() {
  echo "Usage: $0 [start|stop|restart|uninstall]"
  exit 1
}

get_lan_ip() {
  # Detect LAN IP outside proot. FrankenPHP's PATH is only bin/, so PHP
  # cannot exec Termux ifconfig. Prefer Wi-Fi/hotspot over vgate/VPN/rmnet.
  local output=""
  if command -v ifconfig >/dev/null 2>&1; then
    output="$(ifconfig 2>/dev/null || true)"
  elif command -v ip >/dev/null 2>&1; then
    output="$(ip -4 addr show 2>/dev/null || true)"
  fi
  if [ -n "$output" ]; then
    local parsed
    parsed="$(printf '%s\n' "$output" | awk '
      BEGIN { best = -1; skip = 1 }
      function set_iface(name) {
        iface = tolower(name)
        sub(/:$/, "", iface)
        sub(/@.*/, "", iface)
        skip = (iface == "lo" || iface ~ /^(rmnet|tun|wg|ppp|ccmni|pdp|clat|dummy|orichi|sit|ipsec)/)
        score = 40
        if (iface ~ /^(ap[0-9]*|softap[0-9]*)$/ || iface ~ /wlan[0-9]*_ap/) score = 100
        else if (iface ~ /^wlan[0-9]+/) score = 90
        else if (iface ~ /^(rndis|usb|eth|bnep)/) score = 70
        else if (iface ~ /^vgate/) score = 20
      }
      /^[0-9]+:\s+/ { set_iface($2); next }
      /^[A-Za-z0-9_.-]+/ { set_iface($1); next }
      skip { next }
      /inet / {
        for (i = 1; i <= NF; i++) {
          val = $i
          sub(/^addr:/, "", val)
          sub(/\/.*/, "", val)
          split(val, o, ".")
          if (o[1] == 10 || (o[1] == 172 && o[2] >= 16 && o[2] <= 31) || (o[1] == 192 && o[2] == 168)) {
            if (val != "127.0.0.1" && val !~ /^169\.254\./ && val !~ /^172\.17\./ && val !~ /^192\.168\.56\./) {
              if (score > best) { best = score; bestip = val }
            }
          }
        }
      }
      END { if (bestip != "") print bestip }
    ')"
    if [ -n "$parsed" ]; then
      echo "$parsed"
      return 0
    fi
  fi
  if command -v getprop >/dev/null 2>&1; then
    local ip=""
    for prop in dhcp.wlan2.ipaddress dhcp.wlan0.ipaddress dhcp.wlan1.ipaddress dhcp.wlan3.ipaddress dhcp.ap0.ipaddress dhcp.rndis0.ipaddress dhcp.eth0.ipaddress; do
      ip="$(getprop "$prop" 2>/dev/null || true)"
      [ -n "$ip" ] && [ "$ip" != "127.0.0.1" ] && echo "$ip" && return 0
    done
  fi
}

print_urls() {
  local lan_ip
  lan_ip="$(get_lan_ip)"
  echo "  Local:    http://127.0.0.1:$PORT"
  [ -n "$lan_ip" ] && echo "  Network:  http://$lan_ip:$PORT"
}

port_in_use() {
  if command -v curl >/dev/null 2>&1; then
    curl -s -o /dev/null http://127.0.0.1:"$PORT" 2>/dev/null && return 0
  fi
  if command -v wget >/dev/null 2>&1; then
    wget -q -O /dev/null http://127.0.0.1:"$PORT" 2>/dev/null && return 0
  fi
  return 1
}

do_stop() {
  echo "Stopping PencariMovie Server..."

  local pid=""
  for pid_file in "$APP_DIR/.frankenphp.pid" "$APP_DIR/.php-server.pid" "$APP_DIR/.addon.pid"; do
    if [ -f "$pid_file" ]; then
      pid="$(cat "$pid_file" 2>/dev/null || true)"
      if [ -n "$pid" ]; then
        kill "$pid" 2>/dev/null || true
        kill -9 "$pid" 2>/dev/null || true
      fi
      rm -f "$pid_file" 2>/dev/null || true
    fi
  done

  pkill -9 -f "frankenphp.*php-server" 2>/dev/null || true
  pkill -9 -f "php.*router\.php" 2>/dev/null || true
  pkill -9 -f "addon\.js" 2>/dev/null || true
  pkill -9 -f "bin/addon" 2>/dev/null || true
  pkill -9 -f "node.*addon" 2>/dev/null || true
  pkill -9 -f "bun.*addon" 2>/dev/null || true

  if command -v lsof >/dev/null 2>&1; then
    local pids_8089 pids_port
    pids_8089="$(lsof -ti tcp:8089 -sTCP:LISTEN 2>/dev/null || true)"
    [ -n "$pids_8089" ] && kill -9 $pids_8089 2>/dev/null || true
    pids_port="$(lsof -ti tcp:"$PORT" -sTCP:LISTEN 2>/dev/null || true)"
    [ -n "$pids_port" ] && kill -9 $pids_port 2>/dev/null || true
  fi

  if command -v fuser >/dev/null 2>&1; then
    fuser -k -9 "$PORT"/tcp 2>/dev/null || true
    fuser -k -9 8089/tcp 2>/dev/null || true
  fi

  echo "Server stopped."
}

download_file() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -L --fail -o "$dest" "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -O "$dest" "$url"
  else
    echo "Need curl or wget."; exit 1
  fi
}

# Follow GitHub /releases/latest and return the tag (e.g. v1.0.1).
fetch_latest_tag() {
  local loc=""
  if command -v curl >/dev/null 2>&1; then
    loc="$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" 2>/dev/null || true)"
  elif command -v wget >/dev/null 2>&1; then
    loc="$(wget -q --max-redirect=0 --server-response "https://github.com/$REPO/releases/latest" -O /dev/null 2>&1 \
      | awk 'BEGIN{IGNORECASE=1} /^  Location:/{print $2; exit}' | tr -d '\r' || true)"
  fi
  loc="${loc%$'\r'}"
  loc="${loc%/}"
  local tag="${loc##*/}"
  case "$tag" in
    v[0-9]*) echo "$tag" ;;
    *) return 1 ;;
  esac
}

current_tag() {
  if [ -f "$APP_DIR/.release-tag" ]; then
    tr -d '\r\n' < "$APP_DIR/.release-tag"
  fi
}

find_release_root() {
  local extract_dir="$1"
  if [ -f "$extract_dir/backend.php" ] || [ -f "$extract_dir/start.sh" ]; then
    echo "$extract_dir"
    return
  fi
  local found=""
  found="$(find "$extract_dir" -maxdepth 2 -type f \( -name backend.php -o -name start.sh \) 2>/dev/null | head -1 || true)"
  if [ -n "$found" ]; then
    dirname "$found"
    return
  fi
  echo "$extract_dir"
}

# Copy a extracted release into APP_DIR without touching existing storage/.
migrate_legacy_dir() {
  if [ -d "$OLD_APP_DIR" ] && [ "$OLD_APP_DIR" != "$APP_DIR" ]; then
    if [ -d "$OLD_APP_DIR/storage" ] && [ ! -d "$APP_DIR/storage" ]; then
      mkdir -p "$APP_DIR"
      cp -R "$OLD_APP_DIR/storage" "$APP_DIR/storage"
    fi
    rm -rf "$OLD_APP_DIR"
  fi
}

copy_release_into_app() {
  local src="$1" item name
  mkdir -p "$APP_DIR"
  for item in "$src"/*; do
    [ -e "$item" ] || continue
    name="$(basename "$item")"
    if [ "$name" = "storage" ]; then
      mkdir -p "$APP_DIR/storage"
      continue
    fi
    rm -rf "$APP_DIR/$name"
    cp -R "$item" "$APP_DIR/$name"
  done
}

strip_crlf() {
  local dir="${1:-.}" f
  for f in "$dir"/*.sh; do
    [ -f "$f" ] || continue
    tr -d '\r' < "$f" > "$f.tmp" && mv "$f.tmp" "$f"
  done
}

download_extract() {
  local target="$1" tag="$2"
  local url="https://github.com/$REPO/releases/download/$tag/pencarimovie-downloader-$target.tar.gz"
  local tmp src

  tmp="${TMPDIR:-/tmp}/pencarimovie-ota-$$"
  rm -rf "$tmp"
  mkdir -p "$tmp/extract"

  echo "Downloading $url"
  download_file "$url" "$tmp/pencarimovie.tar.gz"
  tar -xzf "$tmp/pencarimovie.tar.gz" -C "$tmp/extract"
  src="$(find_release_root "$tmp/extract")"
  copy_release_into_app "$src"
  strip_crlf "$APP_DIR"
  printf '%s\n' "$tag" > "$APP_DIR/.release-tag"
  rm -rf "$tmp"
}

# Returns 0 if files were installed/updated, 1 if already up to date.
install_or_update() {
  if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
    if command -v pkg >/dev/null 2>&1; then
      echo "Installing curl..."
      pkg install -y curl
    fi
  fi

  local target latest current
  target="$(detect_target)"
  latest="$(fetch_latest_tag || true)"
  current="$(current_tag)"

  if [ -d "$APP_DIR" ] && [ -z "$current" ]; then
    current="$FALLBACK_TAG"
    printf '%s\n' "$current" > "$APP_DIR/.release-tag"
  fi

  if [ -z "$latest" ]; then
    if [ -d "$APP_DIR" ]; then
      echo "Could not check GitHub for updates; using installed copy."
      return 1
    fi
    latest="$FALLBACK_TAG"
  fi

  if [ -d "$APP_DIR" ] && [ "$current" = "$latest" ]; then
    return 1
  fi

  if [ ! -d "$APP_DIR" ]; then
    echo "Downloading PencariMovie Server $latest ($target)..."
  else
    echo "Updating PencariMovie Server ${current:-unknown} -> $latest ($target)..."
    if port_in_use; then
      do_stop
      sleep 1
    fi
  fi

  download_extract "$target" "$latest"
  register_cli
  return 0
}

register_cli() {
  local bin_dir="${PREFIX:-/data/data/com.termux/files/usr}/bin"
  local launcher="$APP_DIR/pencarimovie-termux.sh"

  if [ ! -f "$launcher" ]; then
    cat <<'EOF' > "$launcher"
#!/usr/bin/env bash
dir="$(cd "$(dirname "$0")" && pwd)"
case "${1:-}" in
  stop|--stop)
    bash "$dir/stop.sh"
    ;;
  restart|--restart)
    bash "$dir/restart.sh"
    ;;
  *)
    bash "$dir/start-termux.sh"
    ;;
esac
EOF
  fi
  chmod +x "$launcher" 2>/dev/null || true

  if [ -d "$bin_dir" ]; then
    for cmd in pms pm pencarimovie; do
      cat <<EOF > "$bin_dir/$cmd"
#!/usr/bin/env bash
exec "$launcher" "\$@"
EOF
      chmod +x "$bin_dir/$cmd" 2>/dev/null || true
    done
  fi
}

do_start() {
  migrate_legacy_dir

  local had_app=0 updated=0
  [ -d "$APP_DIR" ] && had_app=1

  if install_or_update; then
    updated=1
  fi

  if port_in_use; then
    register_cli
  
    if [ "$had_app" -eq 1 ] && [ "$updated" -eq 0 ]; then
      echo "Server is already running on port $PORT."
      print_urls
      echo "  CLI:      pms [start|stop|restart]"
      echo "  Stop:     pms stop"
      echo "  Restart:  pms restart"
      return
    fi
    echo "Port $PORT is already in use; stopping leftover process..."
    do_stop
    sleep 1
  fi

  cd "$APP_DIR"
  strip_crlf "."

  # ── Follow start-termux.sh template exactly ──────────────────
  ROOT_DIR="$(pwd)"
  FRANKENPHP_BIN="$ROOT_DIR/bin/frankenphp"
  TMP_DIR="$ROOT_DIR/tmp"
  LOG_FILE="${LOG_FILE:-$ROOT_DIR/frankenphp.log}"
  PID_FILE="$ROOT_DIR/.frankenphp.pid"

  echo "Preparing Termux runtime..."

  mkdir -p "$TMP_DIR"
  chmod 700 "$TMP_DIR" 2>/dev/null || true

  for FILE in \
    "$FRANKENPHP_BIN" \
    "$ROOT_DIR/bin/php" \
    "$ROOT_DIR/bin/addon" \
    "$ROOT_DIR/backend.php" \
    "$ROOT_DIR/index.php" \
    "$ROOT_DIR/router.php" \
    "$ROOT_DIR/start.sh" \
    "$ROOT_DIR/stop.sh" \
    "$ROOT_DIR/restart.sh" \
    "$ROOT_DIR/install.sh" \
    "$ROOT_DIR/start-termux.sh" \
    "$ROOT_DIR/install-termux.sh" \
    "$ROOT_DIR/restart-termux.sh"
  do
    if [ -f "$FILE" ]; then
      chmod u+x "$FILE" 2>/dev/null || true
    fi
  done

  LAN_IP="$(get_lan_ip || true)"
  mkdir -p "$ROOT_DIR/storage"
  if [ -n "${LAN_IP}" ]; then
    printf '%s\n' "$LAN_IP" > "$ROOT_DIR/storage/lan_ip.txt"
  else
    rm -f "$ROOT_DIR/storage/lan_ip.txt"
  fi

  # Start Addon server in background (port 8089)
  ADDON_LOG="$ROOT_DIR/addon.log"
  ADDON_PID_FILE="$ROOT_DIR/.addon.pid"

  echo "Checking Addon service..."
  ADDON_CMD=""
  if [ -f "$ROOT_DIR/addon.js" ]; then
    if command -v bun >/dev/null 2>&1; then
      ADDON_CMD="bun \"$ROOT_DIR/addon.js\""
    elif command -v node >/dev/null 2>&1; then
      ADDON_CMD="node \"$ROOT_DIR/addon.js\""
    elif [ -x "$ROOT_DIR/bin/addon" ] && ! command -v getprop >/dev/null 2>&1; then
      ADDON_CMD="\"$ROOT_DIR/bin/addon\""
    else
      if command -v pkg >/dev/null 2>&1; then
        echo "Installing nodejs and openssl for Addon service..."
        pkg install -y openssl nodejs || true
        if command -v node >/dev/null 2>&1; then
          ADDON_CMD="node \"$ROOT_DIR/addon.js\""
        fi
      fi
    fi
  elif [ -x "$ROOT_DIR/bin/addon" ]; then
    ADDON_CMD="\"$ROOT_DIR/bin/addon\""
  fi

  if [ -n "$ADDON_CMD" ]; then
    echo "Starting Addon service ($ADDON_CMD)..."
    nohup sh -c "$ADDON_CMD" >"$ADDON_LOG" 2>&1 &
    echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
    sleep 1

    if command -v curl >/dev/null 2>&1 && curl -s -m 2 http://127.0.0.1:8089/ >/dev/null 2>&1; then
      echo "✓ Addon service is running on http://127.0.0.1:8089"
    else
      if [ -f "$ADDON_LOG" ]; then
        echo "Notice: Addon service log output:"
        cat "$ADDON_LOG"
      fi
    fi
  else
    echo "Warning: Neither nodejs nor bun found. Addon service not started."
  fi

  local started=0

  # ── Primary: Attempt FrankenPHP with proot (small download size) ───────────
  if [ -x "$FRANKENPHP_BIN" ]; then
    if ! command -v proot >/dev/null 2>&1 && command -v pkg >/dev/null 2>&1; then
      echo "Installing proot for FrankenPHP..."
      pkg install -y proot 2>/dev/null || true
    fi

    if command -v proot >/dev/null 2>&1; then
      echo "Starting PencariMovie Server with FrankenPHP through proot..."
      echo "Log file: $LOG_FILE"

      # Use Unix-optimised php.ini (static build — no dynamic extension loading)
      if [ -f "$ROOT_DIR/bin/php.ini.unix" ]; then
        cp "$ROOT_DIR/bin/php.ini.unix" "$ROOT_DIR/bin/php.ini"
      fi

      proot --link2symlink -0 \
        -w "$ROOT_DIR" \
        -b "$ROOT_DIR:$ROOT_DIR" \
        -b "$TMP_DIR:/tmp" \
        /bin/sh -c 'export PATH="$1/bin:$PATH"; export PHP_BINDIR="$1/bin"; export PHPRC="$1/bin"; export LAN_IP="$6"; exec "$2" php-server --listen "$3:$4" --root "$5"' \
        sh "$ROOT_DIR" "$FRANKENPHP_BIN" "$HOST" "$PORT" "$ROOT_DIR" "${LAN_IP:-}" >>"$LOG_FILE" 2>&1 &
      PID="$!"

      echo "$PID" > "$PID_FILE" 2>/dev/null || true
      sleep 2

      if kill -0 "$PID" 2>/dev/null; then
        started=1
      else
        echo "FrankenPHP/proot failed to run (e.g. Samsung/seccomp restriction)."
        rm -f "$PID_FILE"
      fi
    fi
  fi

  # ── Fallback: Native Termux PHP (no proot) ─────────────────────────────────
  if [ "$started" -eq 0 ]; then
    echo "Falling back to native Termux PHP..."
    if ! command -v php >/dev/null 2>&1 || ! php -m 2>/dev/null | grep -qi '^gd$'; then
      if command -v pkg >/dev/null 2>&1; then
        echo "Installing native Termux PHP and extensions (php, php-gd)..."
        pkg install -y php php-gd
      else
        echo "PHP is missing and pkg is not available."
        exit 1
      fi
    fi

    PHP_LOG="$ROOT_DIR/php-server.log"
    PHP_PID_FILE="$ROOT_DIR/.php-server.pid"
    echo "Starting PencariMovie Server with native Termux PHP (no proot)..."
    echo "Log file: $PHP_LOG"

    export LAN_IP="${LAN_IP:-}"
    nohup php -S "$HOST:$PORT" "$ROOT_DIR/router.php" >>"$PHP_LOG" 2>&1 &
    PID="$!"

    echo "$PID" > "$PHP_PID_FILE" 2>/dev/null || true
    sleep 2

    if ! kill -0 "$PID" 2>/dev/null; then
      echo "Native PHP server exited during startup. Last log lines:"
      tail -n 50 "$PHP_LOG" 2>/dev/null || true
      exit 1
    fi
  fi

  # Pre-spawn persistent MadelineProto IPC workers. Under FrankenPHP, workers
  # spawned from within a web request die when the request ends, so they must
  # be detached here (outside any request) for fd_boot_madeline() to connect
  # to them as IPC clients instead of doing slow full direct-mode boots.
  echo "Warming up IPC workers..."
  if command -v php >/dev/null 2>&1; then
    php "$ROOT_DIR/warmup-ipc.php" >/dev/null 2>&1 || true
  else
    "$ROOT_DIR/bin/php" "$ROOT_DIR/warmup-ipc.php" >/dev/null 2>&1 || true
  fi

  echo ""
  echo "PencariMovie Server is running"
  echo "  Local:    http://127.0.0.1:$PORT"
  if [ -n "${LAN_IP}" ]; then
    echo "  Network:  http://$LAN_IP:$PORT"
  fi
  echo "PID: $PID"
}

do_restart() { do_stop; sleep 1; do_start; }

do_uninstall() {
  echo "Stopping PencariMovie Server..."
  do_stop 2>/dev/null || true

  # Remove CLI wrappers
  local bin_dir="${PREFIX:-/data/data/com.termux/files/usr}/bin"
  for cmd in pms pm pencarimovie; do
    rm -f "$bin_dir/$cmd" 2>/dev/null || true
  done

  # Remove the app directory (storage sessions removed too)
  if [ -d "$APP_DIR" ]; then
    echo "Removing $APP_DIR ..."
    rm -rf "$APP_DIR"
  fi

  echo "PencariMovie Server has been uninstalled."
}

case "${1:-}" in
  start|--start|"") do_start ;;
  stop|--stop) do_stop ;;
  restart|--restart) do_restart ;;
  uninstall|--uninstall) do_uninstall ;;
  *) usage ;;
esac
