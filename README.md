# 🎬 PencariMovie Server

<p align="center">
  <strong>High-speed local media stream resolver & downloader for Stremio, Nuvio, and web browsers.</strong><br>
  100% Plug & Play • Zero Configuration • No Telegram Account or Bot Required
</p>

<p align="center">
  <a href="#-quick-install">Quick Install</a> •
  <a href="#-features">Features</a> •
  <a href="#-stremio--nuvio-setup">Stremio & Nuvio</a> •
  <a href="#-cli-commands">CLI Usage</a> •
  <a href="#-remote-streaming-cloudflare-tunnel">Remote Access</a> •
  <a href="#-open-source--security">Security</a>
</p>

---

## What is PencariMovie Server?

**PencariMovie Server** runs a lightweight streaming engine on your local machine, home server, or Android device. It turns Telegram media into direct, high-speed HTTP streams with instant seeking for **Stremio**, **Nuvio**, or the built-in Netflix-style web player.

- **Zero account setup**: No phone numbers, logins, or bot tokens required.
- **Local & Private**: Streams directly from your machine or home network.
- **Self-updating**: Automatically checks for updates on startup.

---

## Quick Install

Install and start the server with a single command:

#### 📱 Android (APK)

> [**📥 Direct APK Download (telegra.my/apk)**](https://telegra.my/apk) _(Install, tap Start Server, and stream)_

#### 🪟 Windows (10/11)

Run in PowerShell:

```powershell
irm telegra.my/win | iex
```

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

Once started, open the web dashboard in your browser:
👉 **`http://127.0.0.1:8088`** _(or your local LAN IP printed in terminal)_

---

## ✨ Features

- **🔌 Plug & Play**: Ready out of the box with zero configuration or credentials.
- **📺 Stremio & Nuvio Ready**: Built-in addon provider with full catalog and direct `.mp4` stream resolution.
- **🎬 Netflix-Style Web Player**: Built-in dark UI with trending titles, categories, and full search.
- **📡 Multi-Device LAN Sharing**: Share streams across devices on your home Wi-Fi (`http://<LAN-IP>:8088`).
- **☁️ 1-Click Cloudflare Tunnel**: Stream outside your home via a free, instant HTTPS tunnel without opening router ports.
- **🤖 Optional Custom Bot Pooling**: Power users can add multiple personal bot tokens in Settings to load-balance high-concurrency downloads.
- **⚡ Background Service**: System tray support on Windows; background service with wake lock on Android.

---

## 📺 Stremio & Nuvio Setup

### 1. Stremio Setup

1. Open the dashboard at `http://127.0.0.1:8088` and click **Addon / Stremio** in the top navigation.
2. **Local Sync (Recommended)**: Use the built-in Stremio API Sync button to install the addon directly to your Stremio account with 1 click.
3. **Manual / Web**: Copy your manifest link (`http://<LAN-IP>:8088/manifest.json` or HTTPS Tunnel URL) and paste it into the Stremio Addon search bar.

### 2. Nuvio Setup

1. Open the **Nuvio** app on your device (Android TV, tablet, or phone) connected to the same Wi-Fi.
2. Go to **Profile** ➔ **Content & Discovery** ➔ **Addons**.
3. Enter your manifest URL: `http://<YOUR-LAN-IP>:8088/manifest.json` (or visit `http://127.0.0.1:8088/nuvio` to copy it).

---

## CLI Commands (`pms`)

The installer registers a global `pms` command on your system:

```bash
pms start       # Starts the server in the background (checks for updates)
pms stop        # Stops the server and background helper services
pms restart     # Restarts the server
pms tunnel      # Enables Cloudflare Tunnel and prints public HTTPS URLs
pms uninstall   # Completely uninstalls the server and cleans up files
```

_(Works from any terminal on Windows, macOS, Linux, and Termux)._

### Custom Port

By default, the server runs on port `8088`. Override it by setting the `PORT` variable:

- **Linux/macOS/Termux**: `PORT=9090 pms start`
- **Windows (PowerShell)**: `$env:PORT="9090"; pms start`

---

## Remote Streaming (Cloudflare Tunnel)

Need to stream when away from your home Wi-Fi?

1. Open the dashboard (`http://127.0.0.1:8088`) and click **⚙️ Settings**.
2. Under **Cloudflare Tunnel**, click **Enable Tunnel**.
3. Copy the generated public HTTPS URL (e.g. `https://random-words.trycloudflare.com/manifest.json`) and paste it into Stremio or Nuvio.

_No Cloudflare account, domain name, or router port forwarding required._

---

## Open Source & Security

- **Open Source**: Licensed under GPL-3.0. Source code is fully verifiable on GitHub.
- **Local Isolation**: Media requests and tokens are processed locally without third-party middleman servers.
- **Secure Boundaries**: Administrative actions (settings, tunnels, restarts) are restricted to local requests only and blocked on public tunnels.

---

## 🙏 Credits & Acknowledgments

Built on the shoulders of these fantastic open-source projects:

- [**PHP**](https://www.php.net/) — High-performance scripting and asynchronous server execution.
- [**FrankenPHP**](https://github.com/dunglas/frankenphp) — Modern application server built on Go and Caddy.
- [**MadelineProto**](https://github.com/danog/MadelineProto) — Async PHP MTProto client library for Telegram.
- [**cloudflared**](https://github.com/cloudflare/cloudflared) — Cloudflare tunnel client enabling seamless TryCloudflare quick tunnels.
- [**Termux**](https://github.com/termux/termux-app) — Powerful terminal environment for Android devices.

---

<p align="center">
  <sub>Open-source project hosted at <a href="https://github.com/aiskendi/pencarimovie-server">github.com/aiskendi/pencarimovie-server</a></sub>
</p>
