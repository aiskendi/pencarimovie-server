#!/usr/bin/env bash
# ==============================================================================
# PencariMovie Server - Termux Beta Installer
# Downloads the latest release from GitHub, tests standalone native PHP/cURL mode
# without requiring Bun/Node, and starts the server.
# ==============================================================================
set -euo pipefail

print_banner() {
  [ -n "${PENCARIMOVIE_NO_BANNER:-}" ] && return 0
  printf '\033[38;5;208m'
  cat <<'EOF'

 ========================================
    PencariMovie Server (Termux Beta)
 ========================================

EOF
  printf '\033[0m'
}

print_banner

APP_DIR="${HOME:-/data/data/com.termux/files/home}/pencarimovie-server-beta"
PORT="${PORT:-8088}"
HOST="${HOST:-0.0.0.0}"
REPO="aiskendi/pencarimovie-server"

detect_target() {
  local arch os
  arch="$(uname -m)"
  os="$(uname -s)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)  echo "linux-x86_64" ;;
        aarch64|arm64) echo "linux-aarch64" ;;
        *) echo "linux-aarch64" ;;
      esac
      ;;
    *) echo "linux-aarch64" ;;
  esac
}

echo "[1/4] Checking Termux dependencies..."
if command -v pkg >/dev/null 2>&1; then
  echo "Installing required packages (php, curl, openssl, ca-certificates)..."
  pkg install -y php curl openssl ca-certificates 2>/dev/null || true
fi

echo "[2/4] Fetching latest beta / pre-release tag from GitHub ($REPO)..."
TAG="${1:-}"

# Check GitHub Releases (querying /releases allows detecting pre-releases marked as prerelease=true)
if [ -z "$TAG" ] && command -v curl >/dev/null 2>&1; then
  TAG="$(curl -fsSL -H "User-Agent: pencarimovie-beta-installer" "https://api.github.com/repos/$REPO/releases" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4 || true)"
fi

# Fallback to latest stable if /releases listing failed
if [ -z "$TAG" ] && command -v curl >/dev/null 2>&1; then
  TAG="$(curl -fsSL -H "User-Agent: pencarimovie-beta-installer" "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4 || true)"
fi

if [ -z "$TAG" ]; then
  TAG="v1.0.0"
  echo "Using fallback tag: $TAG"
else
  echo "Target release tag: $TAG"
fi

TARGET="$(detect_target)"
TAR_URL="https://github.com/$REPO/releases/download/$TAG/pencarimovie-downloader-$TARGET.tar.gz"

echo "[3/4] Downloading release archive..."
TMP_DIR="${TMPDIR:-/tmp}/pm-beta-$$"
mkdir -p "$TMP_DIR"
TAR_FILE="$TMP_DIR/release.tar.gz"

if curl -fsSL -L -o "$TAR_FILE" "$TAR_URL"; then
  echo "Downloaded $TAR_FILE successfully."
else
  echo "Failed to download from $TAR_URL. Checking if git clone is possible..."
  if command -v git >/dev/null 2>&1; then
    git clone --depth 1 "https://github.com/$REPO.git" "$APP_DIR"
  else
    echo "Error: Unable to fetch release and git is not installed."
    exit 1
  fi
fi

if [ -f "$TAR_FILE" ]; then
  echo "Extracting into $APP_DIR..."
  mkdir -p "$APP_DIR"
  # Release tarballs package files under a top-level directory (pencarimovie-downloader-*)
  # Use --strip-components=1 so backend.php and router.php are extracted directly into $APP_DIR
  if ! tar -xzf "$TAR_FILE" --strip-components=1 -C "$APP_DIR" 2>/dev/null; then
    tar -xzf "$TAR_FILE" -C "$APP_DIR" 2>/dev/null || tar -xf "$TAR_FILE" -C "$APP_DIR"
    # Fallback: if files are nested in a subdirectory, move them up
    SUBDIR="$(find "$APP_DIR" -maxdepth 2 -type f -name "router.php" -exec dirname {} \; 2>/dev/null | head -1 || true)"
    if [ -n "$SUBDIR" ] && [ "$SUBDIR" != "$APP_DIR" ]; then
      cp -rf "$SUBDIR"/* "$APP_DIR/" 2>/dev/null || true
      rm -rf "$SUBDIR" 2>/dev/null || true
    fi
  fi
  rm -rf "$TMP_DIR"
fi

echo "[4/4] Starting PencariMovie Server..."
cd "$APP_DIR"

# Ensure executable permissions on all shell scripts and binaries
chmod +x start.sh start-termux.sh stop.sh restart.sh restart-termux.sh router.php 2>/dev/null || true
[ -f "bin/php" ] && chmod +x "bin/php" 2>/dev/null || true
[ -f "bin/frankenphp" ] && chmod +x "bin/frankenphp" 2>/dev/null || true

# Stop any previous instances
./stop.sh 2>/dev/null || pkill -f "php.*router\.php" 2>/dev/null || true

# Start via start-termux.sh if available, otherwise native php -S with router.php
if [ -f "./start-termux.sh" ]; then
  bash ./start-termux.sh
elif [ -f "./start.sh" ]; then
  bash ./start.sh
else
  PORT="$PORT" HOST="$HOST" nohup php -S "$HOST:$PORT" router.php > "$APP_DIR/server.log" 2>&1 &
  SERVER_PID=$!
  sleep 2
  echo "Server started with PID: $SERVER_PID on http://127.0.0.1:$PORT"
fi
