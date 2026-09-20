#!/usr/bin/env bash
# ==============================================================================
# PencariMovie Server - Docker Installer & Management CLI
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/aiskendi/pencarimovie-server/main/pencarimovie-docker.sh | bash
#   pms-docker [start|stop|restart|logs|update|status|uninstall]
# ==============================================================================
set -euo pipefail

APP_DIR="${APP_DIR:-$HOME/pencarimovie-docker}"
PORT="${PORT:-8088}"
REPO="aiskendi/pencarimovie-server"
GITHUB_RAW="https://raw.githubusercontent.com/$REPO/main"

# ── Docker Detection ─────────────────────────────────────────────────────────
ensure_docker() {
  if command -v docker >/dev/null 2>&1; then
    return 0
  fi

  echo "Docker is not installed."
  if [ "$(id -u)" -eq 0 ] || command -v sudo >/dev/null 2>&1; then
    echo "Installing Docker via official get.docker.com script..."
    curl -fsSL https://get.docker.com | sh
    if [ "$(id -u)" -ne 0 ] && [ -n "${USER:-}" ]; then
      sudo usermod -aG docker "$USER" 2>/dev/null || true
    fi
  else
    echo "Please install Docker first: https://docs.docker.com/engine/install/"
    exit 1
  fi
}

# ── Docker Compose Helper ───────────────────────────────────────────────────
compose_cmd() {
  if docker compose version >/dev/null 2>&1; then
    docker compose "$@"
  elif command -v docker-compose >/dev/null 2>&1; then
    docker-compose "$@"
  else
    return 1
  fi
}

# ── IP Detection ─────────────────────────────────────────────────────────────
get_lan_ip() {
  local ip=""
  if command -v ip >/dev/null 2>&1; then
    ip="$(ip -4 route get 8.8.8.8 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") print $(i+1)}')"
  fi
  if [ -z "$ip" ] && command -v hostname >/dev/null 2>&1; then
    ip="$(hostname -I 2>/dev/null | awk '{print $1}')"
  fi
  if [ -z "$ip" ]; then
    ip="127.0.0.1"
  fi
  echo "$ip"
}

# ── Setup Files ──────────────────────────────────────────────────────────────
setup_files() {
  mkdir -p "$APP_DIR/storage"
  cd "$APP_DIR"

  # Always sync latest docker-compose.yml, Dockerfile, and docker-entrypoint.sh from repository
  curl -fsSL "$GITHUB_RAW/docker-compose.yml" -o docker-compose.yml.new 2>/dev/null && mv docker-compose.yml.new docker-compose.yml || true
  curl -fsSL "$GITHUB_RAW/Dockerfile" -o Dockerfile.new 2>/dev/null && mv Dockerfile.new Dockerfile || true
  curl -fsSL "$GITHUB_RAW/docker-entrypoint.sh" -o docker-entrypoint.sh.new 2>/dev/null && mv docker-entrypoint.sh.new docker-entrypoint.sh && chmod +x docker-entrypoint.sh || true
}

# ── Register CLI Helper ──────────────────────────────────────────────────────
register_cli() {
  local target_script="$APP_DIR/pencarimovie-docker.sh"
  cp "$0" "$target_script" 2>/dev/null || curl -fsSL "$GITHUB_RAW/pencarimovie-docker.sh" -o "$target_script" 2>/dev/null || true
  chmod +x "$target_script" 2>/dev/null || true

  local wrapper='#!/usr/bin/env bash
APP_DIR="'"$APP_DIR"'" exec "'"$target_script"'" "$@"
'

  # Try ~/.local/bin then /usr/local/bin
  local bin_dir="$HOME/.local/bin"
  mkdir -p "$bin_dir" 2>/dev/null || true
  if [ -w "$bin_dir" ]; then
    printf '%s' "$wrapper" > "$bin_dir/pms-docker"
    chmod +x "$bin_dir/pms-docker"
  fi

  if [ -w "/usr/local/bin" ]; then
    printf '%s' "$wrapper" > "/usr/local/bin/pms-docker"
    chmod +x "/usr/local/bin/pms-docker"
  elif command -v sudo >/dev/null 2>&1; then
    printf '%s' "$wrapper" | sudo tee "/usr/local/bin/pms-docker" >/dev/null 2>&1 || true
    sudo chmod +x "/usr/local/bin/pms-docker" 2>/dev/null || true
  fi
}

# ── Actions ──────────────────────────────────────────────────────────────────
do_start() {
  ensure_docker
  setup_files
  cd "$APP_DIR"

  echo "Starting PencariMovie Server in Docker..."
  if compose_cmd up -d --build; then
    :
  else
    echo "Falling back to standalone docker run..."
    docker build -t pencarimovie-server:latest .
    docker rm -f pencarimovie-server 2>/dev/null || true
    docker run -d \
      --name pencarimovie-server \
      --restart unless-stopped \
      -p "${PORT}:8088" \
      -v "$APP_DIR/storage:/app/storage" \
      pencarimovie-server:latest
  fi

  local lan_ip
  lan_ip="$(get_lan_ip)"
  echo ""
  echo "========================================"
  echo "  PencariMovie Server (Docker) Started"
  echo "========================================"
  echo "  Local:    http://127.0.0.1:${PORT}"
  echo "  Network:  http://${lan_ip}:${PORT}"
  echo "  App Dir:  ${APP_DIR}"
  echo ""
  echo "  CLI:      pms-docker [start|stop|restart|logs|update|status]"
  echo "========================================"
}

do_stop() {
  cd "$APP_DIR" 2>/dev/null || true
  echo "Stopping PencariMovie Server container..."
  if compose_cmd down 2>/dev/null; then
    :
  else
    docker stop pencarimovie-server 2>/dev/null || true
  fi
  echo "Stopped."
}

do_restart() {
  do_stop
  sleep 1
  do_start
}

do_logs() {
  cd "$APP_DIR" 2>/dev/null || true
  if compose_cmd logs -f --tail=100 2>/dev/null; then
    :
  else
    docker logs -f --tail=100 pencarimovie-server
  fi
}

do_status() {
  cd "$APP_DIR" 2>/dev/null || true
  if compose_cmd ps 2>/dev/null; then
    :
  else
    docker ps -f name=pencarimovie-server
  fi
}

do_update() {
  ensure_docker
  setup_files
  cd "$APP_DIR"
  echo "Updating PencariMovie Server Docker..."
  curl -fsSL "$GITHUB_RAW/pencarimovie-docker.sh" -o "$APP_DIR/pencarimovie-docker.sh" 2>/dev/null || true
  chmod +x "$APP_DIR/pencarimovie-docker.sh" 2>/dev/null || true
  compose_cmd build --no-cache
  compose_cmd up -d --force-recreate
  docker image prune -f 2>/dev/null || true
  echo "Updated to latest version."
}

do_uninstall() {
  echo "Uninstalling PencariMovie Server Docker..."
  do_stop
  docker rm -f pencarimovie-server 2>/dev/null || true
  docker rmi -f pencarimovie-server:latest 2>/dev/null || true
  rm -f "$HOME/.local/bin/pms-docker" "/usr/local/bin/pms-docker" 2>/dev/null || true
  read -r -p "Delete all data in $APP_DIR? [y/N] " confirm
  if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
    rm -rf "$APP_DIR"
    echo "Removed $APP_DIR."
  fi
  echo "Uninstalled."
}

# ── Main Entrypoint ──────────────────────────────────────────────────────────
main() {
  local action="${1:-install}"
  case "$action" in
    install|start)
      register_cli
      if [ "$action" = "install" ] && (docker ps -a --format '{{.Names}}' 2>/dev/null | grep -q '^pencarimovie-server$'); then
        echo "Existing PencariMovie Server Docker installation detected. Updating..."
        do_update
      else
        do_start
      fi
      ;;
    stop)
      do_stop
      ;;
    restart)
      do_restart
      ;;
    logs)
      do_logs
      ;;
    status)
      do_status
      ;;
    update)
      do_update
      ;;
    uninstall)
      do_uninstall
      ;;
    help|--help|-h)
      echo "Usage: pms-docker [start|stop|restart|logs|status|update|uninstall]"
      ;;
    *)
      echo "Unknown command: $action"
      echo "Usage: pms-docker [start|stop|restart|logs|status|update|uninstall]"
      exit 1
      ;;
  esac
}

main "$@"
