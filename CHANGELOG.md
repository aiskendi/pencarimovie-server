# Changelog

All notable changes to the PencariMovie Server / Downloader project will be documented in this file.

## [Unreleased]

### Added

- **Stremio HTTP API sync**: addon modal can install `http://127.0.0.1:<port>/manifest.json` or `http://<LAN-IP>:<port>/manifest.json` into a Stremio account from the browser (`api.strem.io` login → addonCollectionGet → addonCollectionSet). Credentials never leave the browser. Choose Wi-Fi / LAN vs Localhost; `/manifest.json` names the addon `PencariMovie (Localhost)`, `PencariMovie (Wi-Fi / LAN)`, or `PencariMovie (Cloudflare)` so the address type is visible in Stremio.

- **Optional Cloudflare TryCloudflare tunnel**:
  - Settings can enable/disable an official `cloudflared` quick tunnel (`*.trycloudflare.com`) without a Cloudflare account.
  - Backend routes: `GET /api/tunnel/status`, `POST /api/tunnel/enable`, `POST /api/tunnel/disable`.
  - Windows spawn helper [`tunnel-spawn.ps1`](tunnel-spawn.ps1:1); binary cached in `storage/bin/`.
  - Nuvio addon modal shows a public Cloudflare tunnel manifest URL when the tunnel is running.
  - `stop.bat` / `stop.sh` kill leftover `cloudflared` processes.

### Changed

- Stremio/Nuvio stream objects now follow the official [addon-sdk stream spec](https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/responses/stream.md), [addon-helloworld](https://github.com/Stremio/addon-helloworld), and [AIOStreams](https://github.com/Viren070/AIOStreams) HTTP MP4 shape:
  - Stream `url` is `https://…/api/download/<payload>/<filename>.mp4` so the **full URL** ends with `.mp4`. Stremio Web's HTML5 check is `url.endsWith('.mp4')`; a query-string `?d=` after `.mp4` made it hide streams ("No streams were found") even when the JSON list was populated. Nuvio was already fine.
  - When `/stream` is requested over localhost HTTP and a TryCloudflare tunnel is live, stream URLs use the public HTTPS origin so Stremio Web can play them.
  - Playable streams emit only `name` + `description` + `url` + `behaviorHints` (no extra `title`). Dummy `externalUrl`-only "Updates" rows are no longer mixed into the playable list.
  - `behaviorHints.notWebReady` is omitted for HTTPS `.mp4` (helloworld / AIOStreams); local HTTP / AVI still sets `notWebReady: true`.
  - Manifest `resources` now declare `idPrefixes: ["pm:", "tt"]` on `meta` and `stream` so Stremio 5 / AIOStreams actually query `pm:file:` catalog IDs.
  - Invalid `behaviorHints.headers` removed (SDK uses `proxyHeaders` only with `notWebReady`).
  - `/api/download` no longer overwrites MIME with Telegram `file_type=document`; it serves `video/mp4` (or guessed video MIME) so HTML5 players accept the stream. Query `?d=` remains supported for the dashboard download button.
  - `/api/download` also sends CORS `Access-Control-Allow-Origin: *` plus `Access-Control-Expose-Headers` for Range so Stremio Web can probe the MP4.
- File detail (`#file/SHORT_CODE`) resolves through local `GET /api/resolve-shortcode` instead of calling WordPress `/resolve-file` with a client `api_secret`. Tunneled sessions hide that secret, so public TryCloudflare file pages can now build `/api/download?d=...`.
- Tunnel status prefers the live `cloudflared` `/quicktunnel` hostname over a stale log scrape or `state.json` URL, so Settings does not advertise a hostname that returns Cloudflare Error 1033.
- `cloudflared` is downloaded into `storage/bin/` first; a `PATH` binary is only a fallback.
- Windows spawn writes `--logfile` and `--metrics` so the public URL can be read after handshake.

### Security

- Cloudflare-proxied requests are treated as remote (Host `.trycloudflare.com` / `CF-*` headers), so tunnel enable/disable and bot login stay local-only.
- Tunneled `/api/session` hides `api_secret`. Catalog, stream, and `/api/download` remain reachable through the public URL by design.

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
