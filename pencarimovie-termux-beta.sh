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
  tar -xzf "$TAR_FILE" -C "$APP_DIR" 2>/dev/null || tar -xf "$TAR_FILE" -C "$APP_DIR"
  rm -rf "$TMP_DIR"
fi

echo "[4/4] Starting PencariMovie Server (Beta Native Mode)..."
cd "$APP_DIR"

# Ensure executable permissions
chmod +x start.sh stop.sh router.php 2>/dev/null || true

# Stop previous instances if running on port
pkill -f "php.*router\.php" 2>/dev/null || true

echo "Launching server on http://127.0.0.1:$PORT..."
PORT="$PORT" HOST="$HOST" nohup php -S "$HOST:$PORT" router.php > "$APP_DIR/server-beta.log" 2>&1 &
SERVER_PID=$!
sleep 2

if curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$PORT/manifest.json" | grep -q "200"; then
  echo ""
  echo "============================================================"
  echo " PencariMovie Server (Beta) started successfully! [PID: $SERVER_PID]"
  echo " Local URL:    http://127.0.0.1:$PORT"
  echo " Manifest URL: http://127.0.0.1:$PORT/manifest.json"
  echo " Log file:     $APP_DIR/server-beta.log"
  echo " Stop server:  kill $SERVER_PID"
  echo "============================================================"
else
  echo "Notice: Server started with PID $SERVER_PID. Check $APP_DIR/server-beta.log for details."
fi
