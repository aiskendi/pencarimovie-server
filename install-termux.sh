#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT_DIR"

echo "Installing Termux helpers for PencariMovie Server..."

if command -v pkg >/dev/null 2>&1; then
  # ffmpeg is required for ALAC -> FLAC transcoding. The acoustid/ffmpeg-build
  # binary is glibc-linked and cannot run under Termux's bionic libc, so the
  # native Termux ffmpeg package is used instead.
  echo "Ensuring required Termux packages (proot, openssl, ca-certificates, ffmpeg)..."
  pkg install -y proot openssl ca-certificates ffmpeg 2>/dev/null || true
fi

mkdir -p tmp
chmod 700 tmp 2>/dev/null || true

for FILE in \
  bin/frankenphp \
  bin/php \
  start.sh \
  stop.sh \
  restart.sh \
  install.sh \
  start-termux.sh \
  install-termux.sh \
  restart-termux.sh
do
  if [ -f "$FILE" ]; then
    chmod u+x "$FILE" 2>/dev/null || true
  fi
done

ARCH="$(uname -m 2>/dev/null || echo unknown)"
if [ "$ARCH" != "aarch64" ] && [ "$ARCH" != "arm64" ]; then
  echo "Warning: detected architecture is $ARCH. Most Android phones need the linux-aarch64 package."
fi

if [ ! -f vendor/autoload.php ]; then
  echo "Bundled vendor dependencies were not found; running install.sh..."
  bash install.sh
else
  echo "Bundled vendor dependencies found. Composer install is not needed."
fi

echo "Termux setup complete. Start with: bash start-termux.sh"
