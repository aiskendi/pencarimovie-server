#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TMP_DIR="$DIST_DIR/.build-tmp"

copy_public_root_windows() {
  local dest="$1"

  cp "$ROOT_DIR/backend.php" "$dest/"
  if [ -f "$dest/backend.php" ]; then
    sed -i.bak \
      -e "s|https://pencarimovie.com|https://telegram-webhook.pencarimovie.com|g" \
      -e "s|define('FD_CURL_RESOLVE', getenv('FD_CURL_RESOLVE') ?: '');|define('FD_CURL_RESOLVE', getenv('FD_CURL_RESOLVE') ?: 'telegram-webhook.pencarimovie.com:443:159.195.95.182');|g" \
      "$dest/backend.php" && rm -f "$dest/backend.php.bak"
    php -r '$f=$argv[1];$c=file_get_contents($f);$c=preg_replace("/function fd_log\\(string \\\$message, array \\\$context = \\[\\]\\): void\r?\n\\{.*?\r?\n\\}/s","function fd_log(string \$message, array \$context = []): void {}",$c);file_put_contents($f,$c);' "$dest/backend.php"
  fi
  cp "$ROOT_DIR/index.php" "$dest/"
  cp "$ROOT_DIR/router.php" "$dest/"
  cp "$ROOT_DIR/install.bat" "$dest/"
  cp "$ROOT_DIR/start.bat" "$dest/"
  cp "$ROOT_DIR/restart.bat" "$dest/"
  cp "$ROOT_DIR/stop.bat" "$dest/"
  cp "$ROOT_DIR/tunnel-spawn.ps1" "$dest/"
  cp "$ROOT_DIR/update.ps1" "$dest/"
  cp "$ROOT_DIR/package.json" "$dest/"
  cp "$ROOT_DIR/README.md" "$dest/"
  cp "$ROOT_DIR/LICENSE" "$dest/"
  cp "$ROOT_DIR/SECURITY.md" "$dest/"
  cp -R "$ROOT_DIR/public" "$dest/public"
  mkdir -p "$dest/storage"
  cp "$ROOT_DIR/storage/.gitkeep" "$dest/storage/.gitkeep" 2>/dev/null || true
  cp "$ROOT_DIR/storage/config.example.json" "$dest/storage/config.example.json"
  if [ -f "$ROOT_DIR/storage/catalog_settings.json" ]; then
    cp "$ROOT_DIR/storage/catalog_settings.json" "$dest/storage/catalog_settings.json"
  fi
}

copy_public_root_unix() {
  local dest="$1"

  cp "$ROOT_DIR/backend.php" "$dest/"
  if [ -f "$dest/backend.php" ]; then
    sed -i.bak \
      -e "s|https://pencarimovie.com|https://telegram-webhook.pencarimovie.com|g" \
      -e "s|define('FD_CURL_RESOLVE', getenv('FD_CURL_RESOLVE') ?: '');|define('FD_CURL_RESOLVE', getenv('FD_CURL_RESOLVE') ?: 'telegram-webhook.pencarimovie.com:443:159.195.95.182');|g" \
      "$dest/backend.php" && rm -f "$dest/backend.php.bak"
    php -r '$f=$argv[1];$c=file_get_contents($f);$c=preg_replace("/function fd_log\\(string \\\$message, array \\\$context = \\[\\]\\): void\r?\n\\{.*?\r?\n\\}/s","function fd_log(string \$message, array \$context = []): void {}",$c);file_put_contents($f,$c);' "$dest/backend.php"
  fi
  cp "$ROOT_DIR/index.php" "$dest/"
  cp "$ROOT_DIR/router.php" "$dest/"
  cp "$ROOT_DIR/install.sh" "$dest/"
  cp "$ROOT_DIR/install-termux.sh" "$dest/"
  cp "$ROOT_DIR/install-samsung.sh" "$dest/"
  cp "$ROOT_DIR/start.sh" "$dest/"
  cp "$ROOT_DIR/start-termux.sh" "$dest/"
  cp "$ROOT_DIR/start-samsung.sh" "$dest/"
  cp "$ROOT_DIR/restart.sh" "$dest/"
  cp "$ROOT_DIR/restart-termux.sh" "$dest/"
  cp "$ROOT_DIR/restart-samsung.sh" "$dest/"
  cp "$ROOT_DIR/stop.sh" "$dest/"
  cp "$ROOT_DIR/package.json" "$dest/"
  cp "$ROOT_DIR/README.md" "$dest/"
  cp "$ROOT_DIR/LICENSE" "$dest/"
  cp "$ROOT_DIR/SECURITY.md" "$dest/"
  cp -R "$ROOT_DIR/public" "$dest/public"
  mkdir -p "$dest/storage"
  cp "$ROOT_DIR/storage/.gitkeep" "$dest/storage/.gitkeep" 2>/dev/null || true
  cp "$ROOT_DIR/storage/config.example.json" "$dest/storage/config.example.json"
  if [ -f "$ROOT_DIR/storage/catalog_settings.json" ]; then
    cp "$ROOT_DIR/storage/catalog_settings.json" "$dest/storage/catalog_settings.json"
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
