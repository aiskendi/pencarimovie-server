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
  APP_DIR="${HOME:-/root}/pencarimovie-server"
  OLD_APP_DIR="${HOME:-/root}/pencarimovie-downloader"
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
  local ip=""
  if command -v ip >/dev/null 2>&1; then
    ip="$(ip route get 8.8.8.8 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") {print $(i+1); exit}}')"
  fi
  if [ -z "$ip" ] && command -v hostname >/dev/null 2>&1; then
    ip="$(hostname -I 2>/dev/null | awk '{print $1}')"
  fi
  echo "$ip"
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

  rm -f "$APP_DIR/.frankenphp.pid"
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
  local bin_dir="${HOME:-/root}/.local/bin"
  mkdir -p "$bin_dir" 2>/dev/null || true

  # Ensure launcher exists in APP_DIR
  local launcher="$APP_DIR/pencarimovie-linux.sh"
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
    bash "$dir/start.sh"
    ;;
esac
EOF
  fi
  chmod +x "$launcher" 2>/dev/null || true

  # Install wrappers
  local system_bin="/usr/local/bin"
  for cmd in pms pm pencarimovie; do
    cat <<EOF > "$bin_dir/$cmd"
#!/usr/bin/env bash
exec "$launcher" "\$@"
EOF
    chmod +x "$bin_dir/$cmd" 2>/dev/null || true

    if [ -w "$system_bin" ]; then
      cp -f "$bin_dir/$cmd" "$system_bin/$cmd" 2>/dev/null || true
    elif command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
      sudo cp -f "$bin_dir/$cmd" "$system_bin/$cmd" 2>/dev/null || true
    fi
  done

  # Ensure ~/.local/bin is in PATH in profile files if not already
  for rc in "${HOME:-/root}/.bashrc" "${HOME:-/root}/.profile" "${HOME:-/root}/.bash_profile"; do
    if [ -f "$rc" ] && ! grep -q '\.local/bin' "$rc" 2>/dev/null; then
      printf '\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "$rc"
    fi
  done
  export PATH="${HOME:-/root}/.local/bin:$PATH"

  # Also symlink into $HOME/bin if that directory already exists or is in PATH
  local home_bin="${HOME:-/root}/bin"
  if [ -d "$home_bin" ] || [[ ":$PATH:" == *":${HOME:-/root}/bin:"* ]]; then
    mkdir -p "$home_bin" 2>/dev/null || true
    for cmd in pms pm pencarimovie; do
      ln -sf "$bin_dir/$cmd" "$home_bin/$cmd" 2>/dev/null || cp -f "$bin_dir/$cmd" "$home_bin/$cmd" 2>/dev/null || true
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

  register_cli

  if port_in_use; then
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

  ROOT_DIR="$(pwd)"
  FRANKENPHP_BIN="$ROOT_DIR/bin/frankenphp"

  # Ensure the IPC worker wrapper and runtime are executable. Windows-created
  # tarballs often lose the +x bit, so set it explicitly here.
  for FILE in \
    "$FRANKENPHP_BIN" \
    "$ROOT_DIR/bin/php" \
    "$ROOT_DIR/bin/addon" \
    "$ROOT_DIR/backend.php" \
    "$ROOT_DIR/addon.js" \
    "$ROOT_DIR/index.php" \
    "$ROOT_DIR/router.php" \
    "$ROOT_DIR/start.sh" \
    "$ROOT_DIR/stop.sh" \
    "$ROOT_DIR/restart.sh"
  do
    if [ -f "$FILE" ]; then
      chmod u+x "$FILE" 2>/dev/null || true
    fi
  done

  # Start Addon server in background (port 8089)
  ADDON_LOG="$ROOT_DIR/addon.log"
  ADDON_PID_FILE="$ROOT_DIR/.addon.pid"

  echo "Checking Addon service..."
  local addon_started=0
  if [ -x "$ROOT_DIR/bin/addon" ]; then
    echo "Starting Addon service (bin/addon)..."
    nohup "$ROOT_DIR/bin/addon" >"$ADDON_LOG" 2>&1 &
    echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
    addon_started=1
  elif [ -f "$ROOT_DIR/addon.js" ]; then
    if command -v bun >/dev/null 2>&1; then
      echo "Starting Addon service (bun addon.js)..."
      nohup bun "$ROOT_DIR/addon.js" >"$ADDON_LOG" 2>&1 &
      echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
      addon_started=1
    elif command -v node >/dev/null 2>&1; then
      echo "Starting Addon service (node addon.js)..."
      nohup node "$ROOT_DIR/addon.js" >"$ADDON_LOG" 2>&1 &
      echo $! > "$ADDON_PID_FILE" 2>/dev/null || true
      addon_started=1
    fi
  fi

  if [ "$addon_started" -eq 1 ]; then
    local addon_ready=0
    for _ in 1 2 3 4 5; do
      if command -v curl >/dev/null 2>&1 && curl -s -m 1 http://127.0.0.1:8089/ >/dev/null 2>&1; then
        addon_ready=1
        break
      fi
      sleep 0.4
    done
    if [ "$addon_ready" -eq 1 ]; then
      echo "✓ Addon service is running on http://127.0.0.1:8089"
    elif [ -s "$ADDON_LOG" ]; then
      echo "Notice: Addon service log output:"
      cat "$ADDON_LOG"
    fi
  else
    echo "Warning: Neither bin/addon nor nodejs/bun found. Addon service not started."
  fi

  register_cli
  if [ -t 1 ]; then
    PENCARIMOVIE_NO_BANNER=1 bash start.sh
  else
    # In piped execution (curl | bash), spawn start.sh detached so stdout closes cleanly
    nohup bash -c 'cd "'"$APP_DIR"'" && PENCARIMOVIE_NO_BANNER=1 ./start.sh' >/dev/null 2>&1 &
    sleep 1
    print_urls
    echo "PencariMovie Server started in the background."
  fi
}

do_restart() { do_stop; sleep 1; do_start; }

do_uninstall() {
  echo "Stopping PencariMovie Server..."
  do_stop 2>/dev/null || true

  # Remove CLI wrappers
  local bin_dir="${HOME:-/root}/.local/bin"
  local system_bin="/usr/local/bin"
  local home_bin="${HOME:-/root}/bin"
  for cmd in pms pm pencarimovie; do
    rm -f "$bin_dir/$cmd" 2>/dev/null || true
    rm -f "$system_bin/$cmd" 2>/dev/null || true
    rm -f "$home_bin/$cmd" 2>/dev/null || true
  done

  # Remove the app directory (keeps nothing; storage sessions are removed too)
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
