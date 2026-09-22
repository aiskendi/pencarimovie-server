#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TMP_DIR="$DIST_DIR/.build-tmp"

copy_public_root_windows() {
  local dest="$1"

  for f in Caddyfile backend.php index.php router.php install.bat start.bat stop.bat restart.bat \
           pencarimovie-windows.bat update.ps1 auth-write.ps1 tray.ps1 tray.ico tray.png \
           start-hidden.ps1 tunnel-spawn.ps1 \
           package.json README.md LICENSE SECURITY.md; do
    if [ -f "$ROOT_DIR/$f" ]; then cp "$ROOT_DIR/$f" "$dest/"; fi
  done
  cp -R "$ROOT_DIR/public" "$dest/public"
  if [ -d "$ROOT_DIR/patches" ]; then cp -R "$ROOT_DIR/patches" "$dest/patches"; fi
  mkdir -p "$dest/storage"
  cp "$ROOT_DIR/storage/.gitkeep" "$dest/storage/.gitkeep" 2>/dev/null || true
  cp "$ROOT_DIR/storage/config.example.json" "$dest/storage/config.example.json" 2>/dev/null || true
  # NOTE: storage/catalog_settings.json is intentionally NOT shipped. It is
  # per-user state; shipping the developer's copy would override the built-in
  # defaults in fd_load_catalog_settings() (including the default AIOMetadata
  # upstream manifest) on every fresh install.
  if [ -f "$ROOT_DIR/.release-tag" ]; then
    cp "$ROOT_DIR/.release-tag" "$dest/.release-tag"
  fi
}

copy_public_root_unix() {
  local dest="$1"

  for f in Caddyfile backend.php index.php router.php install.sh install-termux.sh \
           start.sh start-termux.sh restart.sh restart-termux.sh stop.sh \
           pencarimovie-linux.sh pencarimovie-termux.sh pencarimovie-docker.sh \
           package.json README.md LICENSE SECURITY.md; do
    if [ -f "$ROOT_DIR/$f" ]; then cp "$ROOT_DIR/$f" "$dest/"; fi
  done
  cp -R "$ROOT_DIR/public" "$dest/public"
  if [ -d "$ROOT_DIR/patches" ]; then cp -R "$ROOT_DIR/patches" "$dest/patches"; fi
  mkdir -p "$dest/storage"
  cp "$ROOT_DIR/storage/.gitkeep" "$dest/storage/.gitkeep" 2>/dev/null || true
  cp "$ROOT_DIR/storage/config.example.json" "$dest/storage/config.example.json" 2>/dev/null || true
  # NOTE: storage/catalog_settings.json is intentionally NOT shipped. It is
  # per-user state; shipping the developer's copy would override the built-in
  # defaults in fd_load_catalog_settings() (including the default AIOMetadata
  # upstream manifest) on every fresh install.
  if [ -f "$ROOT_DIR/.release-tag" ]; then
    cp "$ROOT_DIR/.release-tag" "$dest/.release-tag"
  fi
}

cleanup() {
  rm -rf "$TMP_DIR"
}

trap cleanup EXIT

mkdir -p "$DIST_DIR"
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR"

build_unix_package() {
  local target="$1"
  local source_file="$2"
  local package_target="${target#frankenphp-}"
  local build_dir="$TMP_DIR/pencarimovie-downloader-$package_target"

  if [ ! -f "$source_file" ]; then
    echo "Missing FrankenPHP source: $source_file"
    exit 1
  fi

  mkdir -p "$build_dir/bin"
  copy_public_root_unix "$build_dir"

  # Unix-optimised php.ini (renamed from .unix to standard .ini)
  cp "$ROOT_DIR/bin/php.ini.unix" "$build_dir/bin/php.ini"
  cp "$ROOT_DIR/bin/php" "$build_dir/bin/php"
  cp "$source_file" "$build_dir/bin/frankenphp"
  chmod +x "$build_dir/bin/frankenphp" "$build_dir/bin/php"
  chmod +x "$build_dir"/*.sh

  if [ -f "$ROOT_DIR/vendor/autoload.php" ]; then
    cp -R "$ROOT_DIR/vendor" "$build_dir/vendor"
  fi

  tar -C "$TMP_DIR" -czf "$DIST_DIR/pencarimovie-downloader-$package_target.tar.gz" "pencarimovie-downloader-$package_target"
}

build_windows_package() {
  local source_zip="$ROOT_DIR/frankenphp-windows-x86_64.zip"
  local build_dir="$TMP_DIR/pencarimovie-downloader-windows-x86_64"
  local extracted="$TMP_DIR/windows-src"

  if [ ! -f "$source_zip" ]; then
    echo "Missing Windows FrankenPHP archive: $source_zip"
    exit 1
  fi

  mkdir -p "$build_dir/bin" "$extracted"
  unzip -q "$source_zip" -d "$extracted"

  copy_public_root_windows "$build_dir"

  # Extract bin/ from the official FrankenPHP Windows release ZIP only
  # Files are extracted to root (not a bin/ subdir), so copy everything
  cp -R "$extracted/"* "$build_dir/bin/"

  # Overlay repo php.ini for Windows runtime (CRITICAL: enables fileinfo, curl, mbstring, openssl, zip DLLs)
  if [ -f "$ROOT_DIR/bin/php.ini" ]; then
    echo "Overlaying repo php.ini for Windows runtime..."
    cp "$ROOT_DIR/bin/php.ini" "$build_dir/bin/php.ini"
  fi

  # Prune unused PHP dev/test binaries that trigger antivirus false positives.
  # php_dl_test.dll is a test-only extension (never loaded in production) and is a
  # well-known generic Riskware/HackTool false-positive magnet. phpdbg/php-cgi/
  # php-win/apache DLLs are also unused by FrankenPHP and only add unsigned EXEs.
  rm -f "$build_dir/bin/ext/php_dl_test.dll" \
        "$build_dir/bin/ext/php_zend_test.dll" \
        "$build_dir/bin/phpdbg.exe" \
        "$build_dir/bin/php8phpdbg.dll" \
        "$build_dir/bin/php-cgi.exe" \
        "$build_dir/bin/php-win.exe" \
        "$build_dir/bin/php8apache2_4.dll" \
        "$build_dir/bin/deplister.exe"
  rm -rf "$build_dir/bin/dev"

  if [ -f "$ROOT_DIR/vendor/autoload.php" ]; then
    cp -R "$ROOT_DIR/vendor" "$build_dir/vendor"
  fi

  (cd "$TMP_DIR" && zip -qr "$DIST_DIR/pencarimovie-downloader-windows-x86_64.zip" "pencarimovie-downloader-windows-x86_64")

  # Publish a SHA-256 sidecar so update.ps1 can verify the download before
  # extracting. This also gives antivirus heuristics a legitimate integrity
  # check to weigh against the download-and-extract pattern.
  (cd "$DIST_DIR" && sha256sum "pencarimovie-downloader-windows-x86_64.zip" | awk '{print $1}' > "pencarimovie-downloader-windows-x86_64.zip.sha256")
}

copy_installer_assets() {
  cp "$ROOT_DIR/pencarimovie-linux.sh" "$DIST_DIR/"
  cp "$ROOT_DIR/pencarimovie-termux.sh" "$DIST_DIR/"
  cp "$ROOT_DIR/pencarimovie-docker.sh" "$DIST_DIR/"
  cp "$ROOT_DIR/pencarimovie-windows.bat" "$DIST_DIR/"
  for f in "$DIST_DIR/pencarimovie-linux.sh" "$DIST_DIR/pencarimovie-termux.sh" "$DIST_DIR/pencarimovie-docker.sh"; do
    tr -d '\r' < "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    chmod +x "$f"
  done
}

build_windows_package
build_unix_package "frankenphp-linux-x86_64" "$ROOT_DIR/frankenphp-linux-x86_64"
build_unix_package "frankenphp-linux-x86_64-gnu" "$ROOT_DIR/frankenphp-linux-x86_64-gnu"
build_unix_package "frankenphp-linux-x86_64-mimalloc" "$ROOT_DIR/frankenphp-linux-x86_64-mimalloc"
build_unix_package "frankenphp-linux-aarch64" "$ROOT_DIR/frankenphp-linux-aarch64"
build_unix_package "frankenphp-linux-aarch64-gnu" "$ROOT_DIR/frankenphp-linux-aarch64-gnu"
build_unix_package "frankenphp-mac-arm64" "$ROOT_DIR/frankenphp-mac-arm64"
build_unix_package "frankenphp-mac-x86_64" "$ROOT_DIR/frankenphp-mac-x86_64"
copy_installer_assets

echo "Release archives created in $DIST_DIR"
