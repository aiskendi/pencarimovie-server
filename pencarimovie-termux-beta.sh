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

APP_DIR="${HOME:-/data/data/com.termux/files/home}/pencarimovie-server"
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

echo "[2/4] Fetching latest release tag from GitHub ($REPO)..."
TAG="${1:-}"

# Check GitHub Releases (detects pre-release or latest release)
if [ -z "$TAG" ] && command -v curl >/dev/null 2>&1; then
  TAG="$(curl -fsSL -H "User-Agent: pencarimovie-beta-installer" "https://api.github.com/repos/$REPO/releases" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4 || true)"
fi

if [ -z "$TAG" ] && command -v curl >/dev/null 2>&1; then
  TAG="$(curl -fsSL -H "User-Agent: pencarimovie-beta-installer" "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4 || true)"
fi

if [ -z "$TAG" ]; then
  TAG="v1.8.0-beta.1"
  echo "Using fallback tag: $TAG"
else
  echo "Target release tag: $TAG"
fi

TARGET="$(detect_target)"
TAR_URL="https://github.com/$REPO/releases/download/$TAG/pencarimovie-downloader-$TARGET.tar.gz"

echo "[3/4] Downloading and extracting release..."
TMP_DIR="${TMPDIR:-/tmp}/pm-beta-$$"
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR/extract"
TAR_FILE="$TMP_DIR/release.tar.gz"

if curl -fsSL -L -o "$TAR_FILE" "$TAR_URL"; then
  echo "Downloaded $TAR_FILE successfully."
else
  echo "Error: Failed to download release from $TAR_URL"
  exit 1
fi

tar -xzf "$TAR_FILE" -C "$TMP_DIR/extract"

# Find release root inside extracted archive
SRC_DIR="$TMP_DIR/extract"
if [ ! -f "$SRC_DIR/router.php" ]; then
  FOUND="$(find "$TMP_DIR/extract" -maxdepth 2 -type f -name "router.php" | head -1 || true)"
  [ -n "$FOUND" ] && SRC_DIR="$(dirname "$FOUND")"
fi

# Copy files directly into $APP_DIR without creating subfolders, preserving storage/
mkdir -p "$APP_DIR"
for item in "$SRC_DIR"/*; do
  [ -e "$item" ] || continue
  name="$(basename "$item")"
  if [ "$name" = "storage" ]; then
    mkdir -p "$APP_DIR/storage"
    continue
  fi
  rm -rf "$APP_DIR/$name"
  cp -rf "$item" "$APP_DIR/$name"
done

printf '%s\n' "$TAG" > "$APP_DIR/.release-tag"
rm -rf "$TMP_DIR"

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
