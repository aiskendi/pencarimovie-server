# Security Policy

## Supported versions

Only the latest published release is supported for security fixes unless a release note says otherwise.

## Reporting a vulnerability

Do not post bot tokens, API hashes, session files, or private URLs in public issues.

If you find a vulnerability, report it privately to the project maintainer through the contact method listed on the GitHub repository. Include:

- affected version or release package name
- operating system
- steps to reproduce
- impact summary
- whether a token, session, or LAN endpoint was exposed

## Local network exposure

PencariMovie Downloader is designed as a local/LAN downloader. The default launcher binds to `0.0.0.0:8088` so devices on the same trusted network can open it.

Do not expose port `8088` directly to the public internet. Anyone who can reach the app may interact with the local downloader endpoints.

## Optional Cloudflare quick tunnel

Settings can start an official Cloudflare TryCloudflare quick tunnel (`cloudflared tunnel --url http://127.0.0.1:<port>`). That publishes a public `https://*.trycloudflare.com` URL to this local server.

Treat that URL as a capability token:

- Anyone with the URL can browse the catalog, resolve files, and stream/download through `/api/download`.
- Tunnel enable/disable, bot login, bot add/remove, and logout stay restricted to the real local dashboard. Cloudflare connections are not treated as localhost even though `cloudflared` proxies as `127.0.0.1`.

## Public mode (`PM_PUBLIC=1`)

Public mode is an explicit opt-in that makes the addon surface multi-user. The security
model changes in four ways:

1. **Addon routes are public.** `/manifest.json`, `/catalog/*`, `/meta/*`,
   `/stream/*`, `/subtitles/*`, `/thumb/*`, `/music/*` (manifest, catalog, meta and
   stream), and `/api/download` no longer require the admin token. The existing per-IP
   counters remain the abuse control.
2. **The admin surface is still closed.** The dashboard is at `/admin`; all management
   APIs still require the password/token, and `auth_enabled()` is forced on even if the
   stored `auth.json` says otherwise.
3. **The admin token is never emitted.** Stream URLs and external links omit the
   `/<token>` segment while public mode is on. Do not paste `/<token>/manifest.json` into
   a public support thread; use `/configure` or `/manifest.json`.
4. **Public links cannot add URLs or loosen filters.** The `<payload>` path segment can
   hide or show built-in catalogs and tighten stream filters. Public links do not use
   upstream addons at all, so the server never fetches a URL because of an anonymous
   public link. Admin manifests and the admin dashboard are unchanged.
5. **`PM_NODES` is a public URL list.** It only affects public stream/download links and
   accepts HTTP(S) origins without credentials, queries, or fragments. Those node URLs are
   emitted to clients; never put an admin token or private hostname in `PM_NODES`.
6. **The local-node option is loopback-only.** `l` accepts only
   `http://127.0.0.1:8088`, `http://localhost:8088` or `http://[::1]:8088`. It is emitted
   as a download URL, never fetched by the public server. A tunneled request is not
   loopback and remains subject to normal auth.
7. **`PM_NODE_PUBLIC_DOWNLOAD=1` opens one route.** It allows `/api/download` without a
   token; it does not open `/`, `/admin`, `/manifest.json`, `/catalog`, `/meta`, `/stream`,
   `/subtitles`, `/music` or any admin API. Use it only on a node reachable by the main
   public host and list it in that host's `PM_NODES`.

Startup guards refuse the default `123456` password and the sample tokens published in
this repository. Set a real `SERVER_PASSWORD`, a fresh `SERVER_TOKEN`, `PM_MAX_STREAMS`
for the host, and `PM_TRUST_XFF=1` behind a reverse proxy.
- Tunneled `/api/session` omits `api_secret`.
- Disable the tunnel from Settings, or run `stop.bat` / `./stop.sh`, when you no longer need public access. Hostnames rotate on each enable.

## Secrets

Treat these files and values as private:

- Telegram bot token
- Telegram API ID/hash if customized
- `storage/config.json`
- `storage/.secret.key`
- MadelineProto session files under `storage/`
- logs that may contain file IDs or request metadata

Never include real secrets in bug reports, screenshots, release ZIPs, or source commits.
