#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: ./scripts/package-unix.sh <target> <frankenphp-binary>"
  echo "Targets: frankenphp-linux-aarch64, frankenphp-linux-aarch64-gnu, frankenphp-linux-x86_64, frankenphp-linux-x86_64-gnu, frankenphp-linux-x86_64-mimalloc, frankenphp-mac-arm64, frankenphp-mac-x86_64"
  exit 1
fi

TARGET="$1"
FRANKENPHP_SOURCE="$2"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

case "$TARGET" in
  frankenphp-linux-aarch64|frankenphp-linux-aarch64-gnu|frankenphp-linux-x86_64|frankenphp-linux-x86_64-gnu|frankenphp-linux-x86_64-mimalloc|frankenphp-mac-arm64|frankenphp-mac-x86_64)
    PACKAGE_TARGET="${TARGET#frankenphp-}"
    ;;
  *)
    echo "Unsupported target: $TARGET"
    exit 1
    ;;
esac

BUILD_DIR="$DIST_DIR/pencarimovie-downloader-$PACKAGE_TARGET"
ARCHIVE="$DIST_DIR/pencarimovie-downloader-$PACKAGE_TARGET.tar.gz"

if [ ! -f "$FRANKENPHP_SOURCE" ]; then
  echo "FrankenPHP binary not found: $FRANKENPHP_SOURCE"
  exit 1
fi

echo "Building $PACKAGE_TARGET release package from $TARGET..."

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/bin" "$DIST_DIR"

# App files
if [ -f "$ROOT_DIR/Caddyfile" ]; then
  cp "$ROOT_DIR/Caddyfile" "$BUILD_DIR/"
fi
cp "$ROOT_DIR/backend.php" "$BUILD_DIR/"
if [ -f "$BUILD_DIR/backend.php" ]; then
  php -r '$f=$argv[1];$c=file_get_contents($f);$c=preg_replace("/function fd_log\\(string \\\$message, array \\\$context = \\[\\]\\): void\r?\n\\{.*?\r?\n\\}/s","function fd_log(string \$message, array \$context = []): void {}",$c);file_put_contents($f,$c);' "$BUILD_DIR/backend.php"
fi
cp "$ROOT_DIR/index.php" "$BUILD_DIR/"
cp "$ROOT_DIR/router.php" "$BUILD_DIR/"
cp "$ROOT_DIR/warmup-ipc.php" "$BUILD_DIR/"
cp "$ROOT_DIR/package.json" "$BUILD_DIR/"
cp "$ROOT_DIR/README.md" "$BUILD_DIR/"
cp "$ROOT_DIR/LICENSE" "$BUILD_DIR/"
cp "$ROOT_DIR/SECURITY.md" "$BUILD_DIR/"

# Unix shell scripts only
cp "$ROOT_DIR/install.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/start.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/stop.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/restart.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/start-termux.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/install-termux.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/restart-termux.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/pencarimovie-linux.sh" "$BUILD_DIR/"
cp "$ROOT_DIR/pencarimovie-termux.sh" "$BUILD_DIR/"

# Public and storage
cp -R "$ROOT_DIR/public" "$BUILD_DIR/public"
if [ -d "$ROOT_DIR/patches" ]; then
  cp -R "$ROOT_DIR/patches" "$BUILD_DIR/patches"
fi
mkdir -p "$BUILD_DIR/storage"
cp "$ROOT_DIR/storage/.gitkeep" "$BUILD_DIR/storage/.gitkeep" 2>/dev/null || true
cp "$ROOT_DIR/storage/config.example.json" "$BUILD_DIR/storage/config.example.json"
if [ -f "$ROOT_DIR/storage/catalog_settings.json" ]; then
  cp "$ROOT_DIR/storage/catalog_settings.json" "$BUILD_DIR/storage/catalog_settings.json"
fi

# Runtime binaries for Unix
cp "$FRANKENPHP_SOURCE" "$BUILD_DIR/bin/frankenphp"
cp "$ROOT_DIR/bin/php" "$BUILD_DIR/bin/php"

# Bundle the audio-encode FFmpeg binary (ALAC -> FLAC conversion).
# Pick the acoustid asset matching the package target's OS/CPU.
case "$PACKAGE_TARGET" in
  mac-arm64)   FFMPEG_ASSET="ffmpeg-8.1.2-audio-encode-arm64-apple-macos11" ;;
  mac-x86_64)  FFMPEG_ASSET="ffmpeg-8.1.2-audio-encode-x86_64-apple-macos10.9" ;;
  *aarch64*)   FFMPEG_ASSET="ffmpeg-8.1.2-audio-encode-arm64-linux-gnu" ;;
  *)           FFMPEG_ASSET="ffmpeg-8.1.2-audio-encode-x86_64-linux-gnu" ;;
esac
FFMPEG_URL="https://github.com/acoustid/ffmpeg-build/releases/download/v8.1.2-1/${FFMPEG_ASSET}.tar.gz"

if [ -f "$ROOT_DIR/storage/bin/ffmpeg" ]; then
  echo "Bundling audio-encode ffmpeg from storage/bin..."
  cp "$ROOT_DIR/storage/bin/ffmpeg" "$BUILD_DIR/bin/ffmpeg"
elif command -v curl >/dev/null 2>&1 || command -v wget >/dev/null 2>&1; then
  echo "Downloading audio-encode ffmpeg ($FFMPEG_ASSET)..."
  FFMPEG_TMP="$(mktemp -d)"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$FFMPEG_URL" -o "$FFMPEG_TMP/ffmpeg.tar.gz" || true
  else
    wget -q "$FFMPEG_URL" -O "$FFMPEG_TMP/ffmpeg.tar.gz" || true
  fi
  if [ -s "$FFMPEG_TMP/ffmpeg.tar.gz" ]; then
    tar -xzf "$FFMPEG_TMP/ffmpeg.tar.gz" -C "$FFMPEG_TMP" 2>/dev/null || true
    FFMPEG_FOUND="$(find "$FFMPEG_TMP" -type f -name ffmpeg | head -n 1)"
    if [ -n "$FFMPEG_FOUND" ]; then
      cp "$FFMPEG_FOUND" "$BUILD_DIR/bin/ffmpeg"
    else
      echo "WARNING: ffmpeg binary not found in archive. ALAC tracks will fall back to a runtime download."
    fi
  else
    echo "WARNING: Failed to download ffmpeg. ALAC tracks will fall back to a runtime download."
  fi
  rm -rf "$FFMPEG_TMP"
else
  echo "WARNING: curl/wget unavailable; skipping ffmpeg bundling. ALAC tracks will fall back to a runtime download."
fi

# Rename php.ini.unix → php.ini for the package (start.sh expects bin/php.ini)
if [ -f "$ROOT_DIR/bin/php.ini.unix" ]; then
  cp "$ROOT_DIR/bin/php.ini.unix" "$BUILD_DIR/bin/php.ini"
else
  echo "WARNING: bin/php.ini.unix not found. Copying bin/php.ini as fallback."
  cp "$ROOT_DIR/bin/php.ini" "$BUILD_DIR/bin/php.ini"
fi

chmod +x "$BUILD_DIR/bin/frankenphp" "$BUILD_DIR/bin/php"
chmod +x "$BUILD_DIR"/*.sh

# Keep packaged shell scripts and bin/php wrapper LF-only. Windows checkouts can copy CRLF,
# which breaks `set -o pipefail` on Linux and causes `syntax error: unexpected 'in'` in /system/bin/sh on Android.
for f in "$BUILD_DIR"/*.sh "$BUILD_DIR/bin/php"; do
  if [ -f "$f" ]; then
    tr -d '\r' < "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    chmod +x "$f"
  fi
done

# Vendor
if [ -f "$ROOT_DIR/vendor/autoload.php" ]; then
  cp -R "$ROOT_DIR/vendor" "$BUILD_DIR/vendor"
else
  echo "WARNING: vendor/autoload.php not found. Release will require Composer install."
fi

tar -C "$DIST_DIR" -czf "$ARCHIVE" "pencarimovie-downloader-$PACKAGE_TARGET"

echo "Created $ARCHIVE"
