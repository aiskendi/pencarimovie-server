# 🎬 PencariMovie Server

<p align="center">
  <strong>Self-hosted Telegram MTProto media stream bridge & local file downloader for Stremio, Nuvio, Eclipse Music, and web clients.</strong><br>
  Local Execution • Direct MTProto Transport • Zero Cloud Relay
</p>

<p align="center">
  <a href="#-quick-install">Quick Install</a> •
  <a href="#-docker--environment-variables">Docker & .env</a> •
  <a href="#-stremio-nuvio--eclipse-music-setup">App Setup</a> •
  <a href="#-server-security--remote-token-auth">Security & Auth</a> •
  <a href="#-cli-commands-pms">CLI Usage</a> •
  <a href="#-features">Features</a> •
  <a href="#-legal-disclaimer--dmca-policy">Legal & DMCA</a>
</p>

---

## 💡 What is PencariMovie Server?

**PencariMovie Server** is a lightweight, self-hosted streaming utility and protocol bridge connecting client applications to Telegram's MTProto API. It converts user-authorized Telegram file references into local HTTP streams with range-request (`HTTP 206`) seek support for media players including **Stremio**, **Nuvio**, and web browsers.

- **Client-Side MTProto Bridge**: Directly streams authorized media to the user's local player using MadelineProto.
- **No Remote File Hosting**: The server does not host, store, or re-transmit copyrighted files; streams originate from Telegram's network directly to the local client.
- **Local & Private**: Runs on the user's own hardware (localhost / private LAN) without passing credentials or media bytes to external proxy servers.
- **Cross-Platform**: Available as a native Windows tray application, Linux daemon, Android APK, Termux script, and Docker container.

---

## 🚀 Quick Install

Launch the server with a single command on your platform of choice:

#### 📱 Android (APK for TV / Phone / Tablet)

> [**📥 Download Android APK (telegra.my/apk)**](https://telegra.my/apk)  
> _(Install on your Android TV, phone, or tablet, tap **Start Server**, and stream)_

#### 🪟 Windows (10 / 11)

Run in **PowerShell**:

```powershell
irm telegra.my/win | iex
```

_(Runs in the background with a System Tray icon. Manage anytime with `pms start` / `pms stop`)._

#### 🐧 Linux

Run in terminal:

```bash
curl -fsSL telegra.my/linux | bash
```

#### 🍏 macOS (Apple Silicon & Intel)

Run in terminal:

```bash
curl -fsSL telegra.my/mac | bash
```

#### 🤖 Android (Termux)

Run in Termux:

```bash
curl -fsSL telegra.my/termux | bash
```

#### 🐳 Docker (Any OS / NAS / VPS)

```bash
docker run -d \
  --name pencarimovie-server \
  --restart unless-stopped \
  -p 8088:8088 \
  -v pencarimovie-data:/app/storage \
  ghcr.io/aiskendi/pencarimovie-server:latest
```

Once started, open the web dashboard:
👉 **`http://127.0.0.1:8088`** _(or your local LAN IP printed in terminal)_

---

## 🐳 Docker & Environment Variables

PencariMovie Server can be fully configured using a `.env` file or container environment variables, allowing you to set static passwords, tokens, ports, and thread limits.

### Docker Compose (`docker-compose.yml`)

```yaml
services:
  pencarimovie:
    image: ghcr.io/aiskendi/pencarimovie-server:latest
    container_name: pencarimovie-server
    restart: unless-stopped
    ports:
      - "${PORT:-8088}:8088"
    env_file:
      - .env
    volumes:
      - ./storage:/app/storage
```

### Environment Variables (`.env`)

Create a `.env` file in your project directory:

```ini
# Server Port
PORT=8088

# Remote Access & Security (Required only when accessed publicly / VPS)
SERVER_PASSWORD=your_secure_password
SERVER_TOKEN=a1b2c3d4e5f67890123456789abcdef0

# Optional: Cloudflare Named Tunnel Token (auto-starts tunnel on launch)
TUNNEL_TOKEN=eyJh...

# Optional: Custom Telegram Bot Token (overrides guest auto-provisioning)
BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz

# Optional: Local Network IP Override
# LAN_IP=192.168.1.100

# Performance & Concurrency Tuning
FRANKENPHP_NUM_THREADS=8
FRANKENPHP_MAX_THREADS=16
FD_DOWNLOAD_PARALLEL_CHUNKS=4

# Optional: External Database for Session ORM (Redis / MySQL / Postgres)
# REDIS_URI=redis://127.0.0.1:6379
# MYSQL_URI=mysql://user:pass@127.0.0.1:3306/pencarimovie

# Debug Mode
DEBUG=0
```

### Configuration Variables Reference

| Variable                      | Default            | Description                                                                                         |
| :---------------------------- | :----------------- | :-------------------------------------------------------------------------------------------------- |
| `PORT`                        | `8088`             | Port where the server listens.                                                                      |
| `SERVER_PASSWORD`             | `123456`           | Password for web dashboard access on public VPS IPs and Cloudflare Tunnels (LAN is password-free).  |
| `SERVER_TOKEN`                | _(auto-generated)_ | 32-character static access token for public remote addon URLs (`/<token>/manifest.json`).           |
| `TUNNEL_TOKEN`                | _(empty)_          | Cloudflare Zero Trust Named Tunnel token. Automatically launches connector on container start.      |
| `BOT_TOKEN`                   | _(auto-provision)_ | Specific Telegram Bot API token. If omitted, the server automatically mints and rotates guest bots. |
| `LAN_IP`                      | _(auto-detected)_  | Manually overrides the host's LAN IPv4 address for multi-device Wi-Fi manifests.                    |
| `FRANKENPHP_NUM_THREADS`      | `8`                | Initial FrankenPHP worker threads for concurrent request handling.                                  |
| `FRANKENPHP_MAX_THREADS`      | `16`               | Maximum FrankenPHP thread scaling ceiling under high stream load.                                   |
| `FD_DOWNLOAD_PARALLEL_CHUNKS` | `4`                | Parallel chunk download concurrency per active stream.                                              |
| `REDIS_URI`                   | _(empty)_          | Redis connection string (`redis://...`) to offload session ORM state.                               |
| `MYSQL_URI`                   | _(empty)_          | MySQL connection string (`mysql://...`) for external database session storage.                      |
| `DEBUG`                       | `0`                | Set to `1` or `true` to enable verbose stream and MadelineProto logging in `storage/debug.log`.     |

---

## 📺 Stremio, Nuvio & Eclipse Music Setup

### 1. Stremio Setup

1. Open the dashboard at `http://127.0.0.1:8088` and click **Addon / Stremio** in the top navigation.
2. **Local Sync (Recommended)**: Click **Install via Stremio API Sync** to push the addon across all your Stremio devices automatically with 1 click.
3. **Manual / Web**: Copy your manifest URL (`http://<LAN-IP>:8088/manifest.json` or `http://<SERVER-IP>:8088/<token>/manifest.json` on public servers) and paste it into Stremio's Addon search box.

### 2. Nuvio Setup

1. Open **Nuvio** on your TV, phone, or tablet connected to the same Wi-Fi.
2. Go to **Profile** ➔ **Content & Discovery** ➔ **Addons**.
3. Enter your manifest URL:
   - **Local Wi-Fi / LAN**: `http://<YOUR-LAN-IP>:8088/manifest.json`
   - **Remote / VPS / Tunnel**: `http://<YOUR-SERVER-IP>:8088/<token>/manifest.json` (or your HTTPS tunnel URL).

### 3. Eclipse Music Setup

1. Open the **Eclipse Music** app (`https://eclipsemusic.app`) on your device.
2. Go to **Settings** ➔ **Connections** ➔ **Add Connection** ➔ **Addon**.
3. Enter your Eclipse manifest URL: `http://<YOUR-LAN-IP>:8088/eclipse/manifest.json` (or `http://127.0.0.1:8088/eclipse`).
4. Search and stream tracks directly with direct seekable playback.

---

## 🔒 Server Security & Remote Token Auth

PencariMovie Server includes built-in security to keep private servers safe when hosted remotely:

- **Local Access is Password-Free**: Requests from localhost (`127.0.0.1`) and private home Wi-Fi (RFC-1918 subnets like `192.168.x.x` or `10.x.x.x`) bypass authentication completely.
- **Public & VPS Protection**: Requests from public IP addresses or Cloudflare Tunnels are protected by password auth (`SERVER_PASSWORD`, default `123456`).
- **Remote Token Routing**: Remote media players authenticate cleanly via path tokens in the manifest URL:
  ```text
  http://<vps-ip>:8088/<token>/manifest.json
  ```
- **CLI Password & Token Management**:
  ```bash
  pms password <new_password>   # Changes server password
  pms reset-password           # Resets password to default (123456)
  pms token                    # Prints current access token
  pms token rotate             # Generates a new access token (invalidates old one)
  ```

---

## 🛠️ CLI Commands (`pms`)

The installer registers a global `pms` command on your system:

```bash
pms start              # Starts server in background (checks for updates)
pms stop               # Stops server and background helper processes
pms restart            # Restarts the server
pms tunnel             # Enables Cloudflare Quick Tunnel (or Named Tunnel if token set)
pms autostart [on|off] # Configures auto-start on system boot
pms password <new>     # Sets the server password
pms reset-password     # Resets password to default (123456)
pms token              # Displays the active 32-character access token
pms token rotate       # Rotates the access token
pms uninstall          # Completely removes server, configuration, and CLI commands
```

_(Works across Windows, macOS, Linux, and Termux)._

---

## ✨ Features

- **🔌 Plug & Play Local Setup**: Runs directly on user machines with auto-configured local session handling.
- **⚡ High-Throughput Streaming**: HTTP 206 range-request support for instant video and audio seeking.
- **📺 Stremio & Nuvio Protocol Compatible**: Implements the Stremio Addon v3 protocol specification for media playback.
- **🎛️ Custom Upstream Addon Bridging**: Easily integrate and toggle custom third-party metadata catalogs.
- **💻 Local Web Player Interface**: Built-in responsive web dashboard for local file access and player playback.
- **📡 Home Network LAN Access**: Access streams on connected TV and mobile devices via local Wi-Fi (`http://<LAN-IP>:8088`).
- **☁️ Optional Secure Tunneling**: Connect remotely using Cloudflare Tunnel integration (`pms tunnel`).
- **🤖 Bot Pool Rotation**: Configure multiple Bot API credentials to balance high-concurrency personal transfers.
- **⚡ System Service Integration**: Windows system tray helper and Android background service support.

---

## ⚖️ Legal Disclaimer & DMCA Policy

### Disclaimer

**PencariMovie Server** is an open-source, non-commercial software tool designed for personal media management, protocol translation, and client-side streaming of user-accessible Telegram files.

1. **No Content Hosting**: This software and its developers **do not host, store, cache, upload, or transmit any media files or copyrighted material**. All media transfers occur directly between the client user, the local software instance, and Telegram's infrastructure.
2. **Third-Party Indexing & Catalogs**: Any metadata, filenames, or search results displayed through client addons or third-party catalog feeds are provided by external sources. The project maintainers do not own, control, or operate third-party Telegram channels or upload services.
3. **User Responsibility**: Users of this software are solely responsible for ensuring that their use complies with local copyright laws, intellectual property regulations, and Telegram's Terms of Service. The developers explicitly disclaim any liability for copyright infringement, unauthorized access, or misuse of this software by third parties.

### Notice & Takedown (DMCA / Copyright Inquiries)

If you are a copyright owner, content producer, or legal representative wishing to report unauthorized references or request exclusion of specific titles from discovery endpoints:

- Please submit a formal written notice detailing:
  1. The copyrighted work claimed to have been infringed;
  2. The specific identification or URL/query where the reference occurs;
  3. Your contact information (name, organization, email, and phone number);
  4. A statement affirming that you hold the copyright or are authorized to act on behalf of the copyright owner.
- Send all DMCA and copyright inquiries to: **`dmca@pencarimovie.com`** (or submit an issue to the repository maintainer).
- Notices are reviewed expeditiously, and qualifying metadata references or index entries will be promptly delisted and excluded.

---

## 🔒 Open Source & Privacy

- **Open Source**: Licensed under GPL-3.0.
- **Local Isolation**: Media requests, streams, and sessions are processed locally without third-party tracking or cloud relays.
- **Secure Boundaries**: Admin actions (settings, bot pool, tunnel triggers) are restricted to local requests only and blocked across public tunnels.

---

## 🙏 Credits & Acknowledgments

Built on the shoulders of these fantastic open-source projects:

- [**PHP**](https://www.php.net/) — Asynchronous server scripting and backend execution.
- [**FrankenPHP**](https://github.com/dunglas/frankenphp) — Modern Go-based PHP application server built on Caddy.
- [**MadelineProto**](https://github.com/danog/MadelineProto) — High-performance async PHP MTProto client library for Telegram.
- [**cloudflared**](https://github.com/cloudflare/cloudflared) — Cloudflare tunnel client enabling seamless TryCloudflare quick tunnels.
- [**Termux**](https://github.com/termux/termux-app) — Terminal environment and process runner for Android devices.

---

<p align="center">
  <sub>Open-source project hosted at <a href="https://github.com/aiskendi/pencarimovie-server">github.com/aiskendi/pencarimovie-server</a></sub>
</p>
