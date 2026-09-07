# 🎬 PencariMovie Server & Downloader

<p align="center">
  <strong>Fast, self-hosted, browser-based Telegram file streaming & downloader server powered by MTProto.</strong>
</p>

<p align="center">
  <a href="#-key-features">Features</a> •
  <a href="#-quick-start--installation">Installation</a> •
  <a href="#-stremio--nuvio-addon-integration">Stremio & Nuvio</a> •
  <a href="#-cloudflare-tunnel">Cloudflare Tunnel</a> •
  <a href="#-how-it-works">How It Works</a> •
  <a href="#-cli-commands">CLI Usage</a> •
  <a href="#-security--privacy">Security</a>
</p>

---

## 💡 Overview

**PencariMovie Server** is a standalone, lightweight local media server that connects directly to Telegram’s MTProto protocol using MadelineProto. It turns Telegram into your personal media streaming and high-speed downloading hub, serving video files through a modern Netflix-style web app or directly into media players like **Stremio** and **Nuvio**.

Everything runs locally on your own machine (PC, Android, TV Box, or Server). No Telegram client installation, no phone number login, and zero third-party dependencies required.

---

## ✨ Key Features

- ⚡ **Direct MTProto Streaming & Chunk Downloader**
  Connects directly to Telegram's data centers via MTProto for maximum download throughput and zero-buffer HTTP byte-range video seeking.

- 📺 **Netflix-Style Web Experience**
  Includes a built-in FlixBrowse dark theme web player with categories, trending keywords, instant search overlay, and detailed media views.

- 🔌 **Full Stremio & Nuvio Addon Integration**
  Out-of-the-box addon server (`/manifest.json` & `/nuvio`). Browse catalogs, discover movies/series, and stream directly in **Stremio** (via In-Browser API Sync or HTTPS Tunnel) and **Nuvio** (via local Wi-Fi / LAN).

- ☁️ **Built-in Cloudflare Quick Tunnel**
  One-click TryCloudflare tunnel built right into Settings. Instantly exposes a secure public HTTPS URL (`*.trycloudflare.com`) without creating a Cloudflare account or configuring port forwarding—perfect for Stremio HTTPS requirements and streaming while away from home.

- 🤖 **Multi-Bot Connection Pooling**
  Add multiple Telegram bot tokens into a connection pool (`/api/bots`) with automatic round-robin balancing to avoid Telegram rate limits and boost concurrent download speeds.

- 📡 **Local Network (LAN) Sharing**
  Auto-detects your LAN IP on startup. Open the web player or configure your TV's addon from any device on your Wi-Fi (`http://192.168.x.x:8088`).

- 🔒 **Privacy-First & Stateless Token Auth**
  No phone number required — log in using standard Telegram Bot Tokens. Tokens are never written to disk in plain text; MadelineProto session files (`storage/`) manage active sessions securely.

- 📦 **Zero-Dependency Standalone Runtime**
  Ships with bundled high-performance **FrankenPHP** runtime. No separate PHP installation, web server, or Node.js required.

- 🔄 **Automatic Start-Time OTA Updates**
  Launchers automatically check GitHub Releases on startup, updating the server code in place while preserving your active bot sessions.

- 🪟 **Windows System Tray Integration**
  Runs cleanly in the background with a system tray icon. Minimize distraction with quick 1-click open, pause, and exit options.

---

## 🚀 Quick Start & Installation

You can install and launch PencariMovie Server in one command using our short domain **`telegra.my`**:

### 1️⃣ Windows (10 / 11)

Open **PowerShell** and run:

```powershell
irm telegra.my/win | iex
```

> **Manual / Alternative:**
>
> ```powershell
> Invoke-RestMethod -Uri "https://raw.githubusercontent.com/aiskendi/pencarimovie-server/main/pencarimovie-windows.bat" -OutFile "pencarimovie-windows.bat"; .\pencarimovie-windows.bat
> ```

- Runs in the background with a **System Tray** helper icon.
- Double-click the tray icon to open the web dashboard.
- Control anytime: `.\pencarimovie-windows.bat [start|stop|restart]`

---

### 2️⃣ Linux (Ubuntu, Debian, Arch, Fedora)

Open your terminal and run:

```bash
curl -fsSL telegra.my/linux | bash
```

> **Manual / Alternative:**
>
> ```bash
> curl -fsSL -o pencarimovie-linux.sh https://raw.githubusercontent.com/aiskendi/pencarimovie-server/main/pencarimovie-linux.sh && bash pencarimovie-linux.sh
> ```

- Control anytime: `pms [start|stop|restart|uninstall]`

---

### 3️⃣ macOS (Apple Silicon M1/M2/M3/M4 & Intel)

Open Terminal and run:

```bash
curl -fsSL telegra.my/mac | bash
```

_(or `curl -fsSL telegra.my/linux | bash` — both automatically detect Apple Silicon or Intel Mac)_

- Control anytime: `pms [start|stop|restart|uninstall]`

---

### 4️⃣ Android via Termux

Recommended for Android devices, TV boxes, or headless micro-servers:

```bash
pkg install openssl curl -y && curl -fsSL telegra.my/termux | bash
```

> **If `curl` fails with a linker error** (e.g. `cannot locate symbol "SSL_set_quic_tls_early_data_enabled"`), your Termux packages are out of sync. `pkg` itself uses `curl`, so repair with low-level `apt` first:
>
> ```bash
> apt update && apt install openssl -y
> ```
>
> Then retry:
>
> ```bash
> curl -fsSL telegra.my/termux | bash
> ```
>
> You can also use `wget` instead of `curl`:
>
> ```bash
> pkg install wget -y && wget -qO- telegra.my/termux | bash
> ```

> **Zero Manual Setup (`proot` handled automatically)**:
>
> - The installer automatically installs `proot` to run FrankenPHP on Android.
> - If `proot` fails (e.g. Samsung One UI / Knox seccomp restrictions), it automatically falls back to native Termux PHP (`php -S`).
> - Use the [Official Termux GitHub Release](https://github.com/termux/termux-app/releases) (Google Play version is outdated). Tap **"More details"** → **"Install anyway"** if prompted by Play Protect.
>
> **Manual / Alternative:**
>
> ```bash
> curl -fsSL -o pencarimovie-termux.sh https://raw.githubusercontent.com/aiskendi/pencarimovie-server/main/pencarimovie-termux.sh && bash pencarimovie-termux.sh
> ```

- Control anytime: `pms [start|stop|restart]`

---

### 5️⃣ Android Standalone App (APK)

Install the standalone Android application with built-in background service and native process manager:

- **Direct Download Shortcut**: **[📥 telegra.my/apk](https://telegra.my/apk)** (or [GitHub Releases](https://github.com/aiskendi/pencarimovie-server/releases/latest))

> ⚠️ **Google Play Protect Notice**:
> Tap **"More details"** and select **"Install anyway"** if warned by Play Protect.

---

## ⚙️ Initial Setup & Bot Authentication

Once the server is running, open your browser at:
👉 **`http://127.0.0.1:8088`** _(or your device's LAN IP)_

1. **Obtain a Bot Token**:
   - Create a free bot in seconds via [@CreateNewTelegramBot](https://t.me/CreateNewTelegramBot?start=localserver).
2. **Connect**:
   - Paste the bot token into the setup gate prompt and click **Connect**.
3. **Session Created**:
   - The backend validates the bot, establishes encrypted MTProto communication, and persists the session.
4. **(Optional) Add Bot Pool**:
   - Add extra bot tokens in Settings to enable multi-bot load balancing and faster parallel streams.

---

## 📺 Stremio & Nuvio Addon Integration

PencariMovie Server includes a built-in addon server fully compatible with both **Stremio** and **Nuvio**.

### 🌟 1. Installing in Stremio

Stremio Web and modern Stremio clients enforce HTTPS and block pasting raw `http://` addon URLs into the search bar. We provide two easy methods to install:

#### Method A: Stremio API Sync (Recommended for Local/LAN)

1. Open the PencariMovie web dashboard at `http://127.0.0.1:8088` and click the **Addon / Stremio** icon in the navbar.
2. Under **Install via Stremio API Sync**, select **Wi-Fi / LAN** or **Localhost**.
3. Enter your Stremio email and password (or paste your Stremio Auth Key).
4. Click **Install via Stremio API Sync**. The browser sends the sync request directly to `https://api.strem.io`—your credentials are never stored or seen by the server.
5. Restart Stremio on your TV, phone, or desktop, and PencariMovie will appear in your addon list!

#### Method B: Cloudflare Tunnel (HTTPS Manifest URL)

1. Enable **Cloudflare Tunnel** in Settings (see below).
2. Copy the generated HTTPS manifest URL (e.g., `https://<subdomain>.trycloudflare.com/manifest.json`).
3. Paste the HTTPS URL directly into Stremio's Addon search box on any device or network and click **Install**.

---

### 🌟 2. Installing in Nuvio

Nuvio natively supports local HTTP manifests:

1. Connect your device (Android TV, phone, tablet) to the **same Wi-Fi / LAN network** as this server.
2. Open the **Nuvio** app on your device.
3. Go to **Profile** ➔ **Content & Discovery** ➔ **Addons**.
4. In the Addon URL field, enter your server's Manifest URL:
   - **Local device**: `http://127.0.0.1:8088/manifest.json`
   - **Other devices on Wi-Fi (TV/Tablet)**: `http://<YOUR-LAN-IP>:8088/manifest.json`
   - **Remote / Mobile Data**: Use your Cloudflare Tunnel HTTPS manifest URL.
5. Click **Install / Add**.

_(You can also visit `http://127.0.0.1:8088/nuvio` on your browser for one-click URL copying and live diagnostics)._

---

## ☁️ Cloudflare Tunnel

Need to stream to Stremio or Nuvio when away from home, or need a valid HTTPS manifest? PencariMovie Server includes a built-in **TryCloudflare** quick tunnel runner:

- **Zero Configuration**: No Cloudflare account, domain name, or port forwarding required.
- **Automatic Setup**: The server automatically downloads the official `cloudflared` binary into `storage/bin/` on first use.
- **How to Enable**:
  1. Open the local dashboard (`http://127.0.0.1:8088`) and click **⚙️ Settings**.
  2. Scroll to **Cloudflare Tunnel** and click **Enable Tunnel**.
  3. Copy your live `https://*.trycloudflare.com` URL to use anywhere!
- **Security Boundaries**: Administrative actions (bot login/logout, adding tokens, starting/stopping the tunnel) are restricted to local requests. External visitors can only stream and browse media.

---

## 🛠️ Easy CLI Commands (Start / Stop / Restart / Uninstall)

Managing the server is as simple as running `start`, `stop`, `restart`, or `uninstall`:

| Platform    | Start       | Stop       | Restart       | Uninstall       | Shortcut / Global CLI                           |
| :---------- | :---------- | :--------- | :------------ | :-------------- | :---------------------------------------------- |
| **Windows** | `pms start` | `pms stop` | `pms restart` | `pms uninstall` | `pms` / `pm` (works anywhere in CMD/PowerShell) |
| **Linux**   | `pms start` | `pms stop` | `pms restart` | `pms uninstall` | `pms` / `pm` (works anywhere in bash/zsh)       |
| **macOS**   | `pms start` | `pms stop` | `pms restart` | `pms uninstall` | `pms` / `pm` (works anywhere in zsh/bash)       |
| **Termux**  | `pms start` | `pms stop` | `pms restart` | `pms uninstall` | `pms` / `pm` (works anywhere in Termux)         |

The installers automatically register `pms` (PencariMovie Server), `pm`, and `pencarimovie` into your environment `$PATH`, allowing you to manage the background server from **any directory** without needing to know the installation folder:

```bash
pms start       # Starts the server in the background
pms stop        # Stops the server and clean up processes
pms restart     # Restarts the server
pms uninstall   # Stops the server, removes CLI commands, and deletes the app folder
```

> ⚠️ **`pms uninstall`** stops the server, removes the `pms`/`pm`/`pencarimovie` commands, and deletes the entire installation folder (including saved bot sessions in `storage/`). This cannot be undone.

_(Legacy `.\pencarimovie-windows.bat`, `--start`, `--stop`, and `--restart` flags remain supported as well)._

### Custom Port Configuration

By default, the server binds to port `8088`. You can change the port using the `PORT` environment variable:

```bash
# Linux / Termux
PORT=9090 bash pencarimovie-linux.sh

# Windows Command Prompt
set PORT=9090 && .\pencarimovie-windows.bat

# Windows PowerShell
$env:PORT="9090"; .\pencarimovie-windows.bat
```

---

## 🏗️ How It Works

```
┌────────────────────────────────────────────────────────┐
│      Clients: Browser / Stremio / Nuvio / Players      │
└───────────────────────────┬────────────────────────────┘
                            │ HTTP / HTTPS (Tunnel)
                            ▼
┌────────────────────────────────────────────────────────┐
│  FrankenPHP Server (Caddy-powered, Embedded PHP 8.2+)  │
│  - backend.php (API Routing & MadelineProto Controller)│
│  - public/ (FlixBrowse Netflix-Style UI & Assets)      │
│  - /manifest.json (Stremio & Nuvio v3 Addon Server)    │
│  - Cloudflare Tunnel Manager (storage/bin/cloudflared) │
└──────────────┬──────────────────────────┬──────────────┘
               │                          │
               ▼                          ▼
┌──────────────────────────────┐ ┌───────────────────────┐
│   WordPress REST / AJAX API  │ │ Telegram Data Centers │
│  (Metadata, Catalog, Search) │ │ (MTProto Protocol via │
│                              │ │  MadelineProto Engine)│
└──────────────────────────────┘ └───────────────────────┘
```

1. **Discovery & Metadata**: The frontend and addon endpoints fetch rich media catalogs, posters, and file references.
2. **On-Demand Resolution**: Bot API file identifiers (`file_id_mt` / `short_code`) are resolved on demand.
3. **High-Speed MTProto Stream**: [`backend.php`](backend.php:1) invokes MadelineProto's [`downloadToBrowser()`](backend.php:4223) with `.mp4` stream paths to deliver direct, seekable video bytes to Stremio, Nuvio, and the web player.

---

## 🔒 Security & Privacy

- **No Plaintext Token Storage**: Bot tokens are used only during initial handshake and are never saved to disk.
- **Local Isolated Sessions**: MTProto session state is safely maintained inside the local [`storage/`](storage/) directory.
- **Protected Endpoints**: Administrative functions (token login, bot management, tunnel configuration) require local network origin and are blocked over external tunnels.
- **Private Network Only**: Run the server on a trusted local network, or use the optional Cloudflare Tunnel when remote streaming is desired.

---

## 🙏 Credits & Acknowledgments

Special thanks to the open-source projects and technologies that make this server possible:

- [**PHP**](https://www.php.net/) — The powerful scripting and server-side language powering the backend logic and async execution.
- [**FrankenPHP**](https://github.com/dunglas/frankenphp) — Modern PHP app server written in Go with Caddy integration.
- [**MadelineProto**](https://github.com/danog/MadelineProto) — High-performance async PHP MTProto client for Telegram.
- [**cloudflared**](https://github.com/cloudflare/cloudflared) — Official Cloudflare tunnel client used for optional TryCloudflare quick tunnels.
- [**Termux**](https://github.com/termux/termux-app) — Android terminal emulator and Linux environment.

---

<p align="center">
  <sub>Built with ❤️ for private, high-speed streaming.</sub>
</p>
