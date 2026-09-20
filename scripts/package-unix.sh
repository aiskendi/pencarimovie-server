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

# No FFmpeg is bundled. Audio is served as-is (AAC/MP3/FLAC/ALAC decoded natively by clients).

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
