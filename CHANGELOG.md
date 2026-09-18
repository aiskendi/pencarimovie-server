# Changelog

All notable changes to the PencariMovie Server / Downloader project will be documented in this file.

## [2.1.8] - 2026-09-18

### Added

- **Server Password Authentication & Token Model**:
  - Remote and public requests (VPS public IP, Cloudflare Tunnel) are guarded by password authentication (default `123456`) and persistent 32-character tokens.
  - Manifest and stream paths support clean token URL routing (`/<token>/manifest.json`, `/<token>/stream/...`).
  - Localhost and private LAN (RFC-1918) requests remain password-free with instant access.
  - CLI management commands: `pms password <new>`, `pms reset-password`, `pms token`, and `pms token rotate` (implemented in [`pencarimovie-linux.sh`](pencarimovie-linux.sh:1), [`pencarimovie-termux.sh`](pencarimovie-termux.sh:1), [`pencarimovie-windows.bat`](pencarimovie-windows.bat:1), and [`auth-write.ps1`](auth-write.ps1:1)).
- **Cloudflare Zero Trust Named Tunnel & Token Persistence**:
  - Support for custom Named Tunnel tokens via Settings UI and `POST /api/tunnel/enable`.
  - Added CLI command `pms tunnel <TOKEN>` across Linux, Windows, and Termux launchers.
  - Dedicated token persistence in `storage/tunnel/token.txt` preserved across server restarts and `pms stop`.
  - Automatic tunnel warm-up in [`warmup-ipc.php`](warmup-ipc.php:1) on `pms start` or `pms restart` when previously enabled.
  - Dynamic discovery of mapped public hostnames matching local listening port from connector logs.
- **VPS & Remote Server Manifest Mode**:
  - Direct IP detection in frontend (`_isIpHost`) preventing public VPS addresses from showing TryCloudflare tunnel badges when tunnels are disabled.
  - Added `mode=server` to [`backend.php`](backend.php:6102) yielding `PencariMovie (Server)` / `org.pencarimovie.addon.server`.
  - Addon modal displays "🌐 Server Manifest" on VPS IPs and automatically selects the Server manifest in Stremio Sync.
  - Dynamically hides the Wi-Fi/LAN vs Localhost toggle when accessing from a public VPS IP or Cloudflare Tunnel.
- **Eclipse Music Addon Integration**:
  - Native discovery and streaming for 500,000+ audio tracks under `/eclipse` routes (`/eclipse/manifest.json`, `/eclipse/search`, `/eclipse/stream/{id}`, `/eclipse/catalog/{id}`, `/eclipse/resolve`).
  - Advanced caption and title parsing in [`backend.php`](backend.php:8165) separating artist, title, and featured collaborations with multi-word connector handling.
- **Stremio HTTP API Sync**:
  - Browser-side Stremio HTTP API sync (`api.strem.io` login → `addonCollectionGet` → `addonCollectionSet`) without transmitting user credentials to the PHP backend.

### Security

- **Caddy Sensitive File Shield**:
  - Added `@blocked` route matcher in [`Caddyfile`](Caddyfile:1) returning HTTP 404 for sensitive files and directories (`storage/*`, `vendor/*`, `bin/*`, `patches/*`, `Caddyfile*`, `*.sh`, `*.bat`, `*.key`, `*.log`, `*.lock`, `*.md`, `*.yml`, `*.yaml`, `composer.json`, `package.json`).
- **Progressive Brute-Force Rate Limiting**:
  - Exponential lockout ladder (30s, 2m, 10m, 30m) for failed password submissions tracked in `storage/cache/auth_lockout.json` returning HTTP 429 with `Retry-After`.
- **Locked Stream Responses**:
  - Unauthenticated remote stream requests receive an informative Stremio stream banner directing users to authenticate via `#addon`, rather than failing silently with 401.
- **Native PHP Static Boundary**:
  - Direct built-in PHP server execution (`php -S ... router.php`) strictly isolates file serving to `public/` via `realpath()` and `str_starts_with()` checks.

### Changed

- **Stream Object Specification**:
  - Playable `/stream` objects comply with AIOStreams specification (`name`, `description`, `url`, `behaviorHints`).
  - Stream URLs end with clean file extensions (`.mp4`, `.flac`, `.mkv`) to guarantee playback compatibility across Stremio Web, Android, and desktop players.
- **Release Packaging**:
  - Updated [`scripts/build-release.bat`](scripts/build-release.bat:1) and [`scripts/package-unix.sh`](scripts/package-unix.sh:1) to include [`pencarimovie-linux.sh`](pencarimovie-linux.sh:1), [`pencarimovie-termux.sh`](pencarimovie-termux.sh:1), [`README.md`](README.md:1), and [`patches/`](patches/) across all release archives.
  - Added automated line ending normalization (CRLF → LF) for all shell scripts and `bin/php` when building Unix packages on Windows.
  - Exported [`update.ps1`](update.ps1:1) to `dist/` alongside [`pencarimovie-windows.bat`](pencarimovie-windows.bat:1).
- **Process Management**:
  - Preserved tunnel token state during `stop.sh` and `stop.bat` process teardown.
  - Improved working directory resolution in Unix start scripts ensuring Caddy directives resolve from the repository root.

## [1.0.1] - 2026-08-29

### Added

- **Stremio & Nuvio Addon Integration**:
  - Implemented Stremio addon manifest and endpoints (`/manifest.json`, `/catalog/...`, `/meta/...`, `/stream/...`) in [`backend.php`](backend.php:2585).
  - Enables direct playback and library browsing from Stremio and Nuvio players with automatic Telegram bot stream resolution.
- **Windows System Tray Helper**:
  - Added background tray management scripts ([`tray.ps1`](tray.ps1:1), [`start-hidden.ps1`](start-hidden.ps1:1)) and tray icons ([`tray.ico`](tray.ico), [`tray.png`](tray.png)).
  - Runs FrankenPHP silently with hidden window and provides tray menu actions for opening the web dashboard and stopping the server.
- **One-File Installers with Start-Time OTA Updates**:
  - Added [`pencarimovie-windows.bat`](pencarimovie-windows.bat:1), [`pencarimovie-linux.sh`](pencarimovie-linux.sh:1), and [`pencarimovie-termux.sh`](pencarimovie-termux.sh:1).
  - Automatically queries GitHub `releases/latest` on start, updates the application files in-place while preserving `storage/` bot session data.
- **Version Check & Minimum Version Enforcement**:
  - Implemented version verification against WordPress API endpoint (`/fastdownloader/v1/version`) with local hourly caching.
  - Added full-screen update notification overlay in [`public/index.html`](public/index.html:27) and [`public/styles.css`](public/styles.css:681).
- **LAN IP Detection & Network Access**:
  - Cross-platform network IP discovery in [`start.bat`](start.bat:1), [`start.sh`](start.sh:1), and [`start-termux.sh`](start-termux.sh:1) displaying local and LAN network URLs upon startup.

### Changed

- **Frontend Streaming UI Overhaul**:
  - Integrated Netflix-style FlixBrowse UI directly into [`public/index.html`](public/index.html:1), [`public/app.js`](public/app.js:1), and [`public/styles.css`](public/styles.css:1).
  - Added rich search overlays, file detail views, inline Telegram stream player, and one-click MadelineProto downloader.
- **Backend Architecture & Security**:
  - Session-based authentication with automatic Telegram bot login via MadelineProto.
  - Eliminated plaintext configuration files; API credentials are securely fetched from the WordPress backend and encrypted with the bot token.
  - Enhanced FrankenPHP storage directory resolution to protect sessions in temp directory environments.
- **Launcher Scripts**:
  - Refactored [`start.bat`](start.bat:1), [`start.sh`](start.sh:1), [`start-termux.sh`](start-termux.sh:1), [`stop.bat`](stop.bat:1), [`stop.sh`](stop.sh:1), [`restart.bat`](restart.bat:1), [`restart.sh`](restart.sh:1), and [`restart-termux.sh`](restart-termux.sh:1) for unified port handling (default `8088`), clean output banners, and process lifecycle management.
- **Project Documentation**:
  - Updated [`README.md`](README.md:1) and [`RELEASE.md`](RELEASE.md:1) with Stremio setup guides, one-line installer commands, and packaging specifications.
