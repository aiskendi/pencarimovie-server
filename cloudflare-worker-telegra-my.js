/**
 * Cloudflare Worker for telegra.my
 *
 * Two jobs:
 *   1. DPI-firewall fallback: reverse-proxies /wp-json/* and /wp-admin/* to
 *      pencarimovie.com over Cloudflare's backbone, so a network that resets the
 *      TLS SNI for that hostname can still read the origin.
 *   2. Short install links for the NATIVE server (port/rust): one binary per OS,
 *      published as pencarimovie-server-<os>-<arch>.tar.gz on releases/latest.
 *      No PHP, no Composer, no bundled runtime - the scripts only download,
 *      verify and extract.
 *
 *   /win     Windows (PowerShell, piped to iex)
 *   /linux   Linux x86_64
 *   /mac     macOS (arm64 is the published build)
 *   /android Android APK download (alias /apk)
 *   /docker  Container image
 *   /logo.png The app logo, proxied so a blocked network still shows it
 *
 * Asset names come from .github/workflows/release.yml (native-build).
 */

const GITHUB_TARGET = "aiskendi/pencarimovie-server";
const RAW_BASE = `https://raw.githubusercontent.com/${GITHUB_TARGET}/main`;
const RELEASE_BASE = `https://github.com/${GITHUB_TARGET}/releases/latest/download`;
const RELEASE_PAGE = `https://github.com/${GITHUB_TARGET}/releases/latest`;
// The Android app is NOT on releases/latest: it ships on the rolling tag
// apk-server-latest (published by android/release-rust.ps1).
const APK_URL = `https://github.com/${GITHUB_TARGET}/releases/download/apk-server-latest/pencarimovie-server_release_universal.apk`;
const IMAGE = `ghcr.io/${GITHUB_TARGET}:latest`;

/* ── Linux / macOS ────────────────────────────────────────────────────────── */

const SH_INSTALL = `#!/bin/sh
# PencariMovie Server - installer for Linux and macOS.
# Downloads the single binary for this machine, verifies its published SHA-256
# and extracts it over ~/pencarimovie-server, keeping storage/ (your bot
# session) untouched. Usage:  curl -fsSL telegra.my/linux | sh
set -eu

REPO="${GITHUB_TARGET}"
BASE="${RELEASE_BASE}"

os=$(uname -s)
arch=$(uname -m)
case "$os" in
  Linux)  os_key=linux ;;
  Darwin) os_key=macos ;;
  *) echo "Unsupported OS: $os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64)  arch_key=x86_64 ;;
  arm64|aarch64) arch_key=aarch64 ;;
  *) echo "Unsupported CPU: $arch" >&2; exit 1 ;;
esac

asset="pencarimovie-server-$os_key-$arch_key.tar.gz"
url="$BASE/$asset"

probe=$(curl -fsSI "$url" 2>/dev/null | head -n 1 || true)
case "$probe" in
  *200*|*302*) ;;
  *) echo "No published build for $os_key/$arch_key yet."
     echo "Other builds: $RELEASE_PAGE"
     exit 1 ;;
esac

dir="$HOME/pencarimovie-server"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "Downloading $asset"
curl -fsSL "$url" -o "$tmp/$asset"
if curl -fsSL "$url.sha256" -o "$tmp/sha" 2>/dev/null; then
  want=$(cut -d' ' -f1 "$tmp/sha")
  if command -v sha256sum >/dev/null 2>&1; then
    got=$(sha256sum "$tmp/$asset" | cut -d' ' -f1)
  else
    got=$(shasum -a 256 "$tmp/$asset" | cut -d' ' -f1)
  fi
  if [ "$want" != "$got" ]; then
    echo "Checksum mismatch - nothing was installed." >&2
    exit 1
  fi
  echo "SHA-256 OK"
fi

tar -xzf "$tmp/$asset" -C "$tmp"
rel="$tmp/pencarimovie-server-$os_key-$arch_key"
mkdir -p "$dir"
# Merge instead of replace: storage/ holds the bot session and must survive.
( cd "$rel" && tar -cf - . ) | ( cd "$dir" && tar -xf - )
chmod +x "$dir/pencarimovie-server" 2>/dev/null || true
for f in "$dir"/*.sh "$dir"/*.command; do
  [ -f "$f" ] && chmod +x "$f" 2>/dev/null || true
done

echo "Installed to $dir"
echo "Starting the server..."
if [ -x "$dir/start.sh" ]; then
  exec "$dir/start.sh"
fi
exec "$dir/pencarimovie-server"
`;

/* ── Windows ──────────────────────────────────────────────────────────────── */

const PS1_INSTALL = `# PencariMovie Server - Windows installer & launcher.
# One binary, no PHP and no bundled runtime, so the only requirement is the
# Visual C++ 2015-2022 x64 runtime that the MSVC build links against.
# Usage:  irm telegra.my/win | iex
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$repo = '${GITHUB_TARGET}'
$base = '${RELEASE_BASE}'
$asset = 'pencarimovie-server-windows-x86_64.tar.gz'
$homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $HOME }
$installDir = Join-Path $homeDir 'pencarimovie-server'

if (-not (Get-Command tar.exe -ErrorAction SilentlyContinue)) {
    Write-Host 'tar.exe is missing. Windows 10 1803+ and Windows 11 include it.' -ForegroundColor Red
    exit 1
}
$vcrDll = Join-Path (Join-Path $env:SystemRoot 'System32') 'vcruntime140.dll'
if (-not (Test-Path -LiteralPath $vcrDll)) {
    Write-Host 'Microsoft Visual C++ 2015-2022 x64 runtime is missing.' -ForegroundColor Red
    Write-Host 'Install it, then re-run: irm telegra.my/win | iex' -ForegroundColor Yellow
    Write-Host 'https://aka.ms/vs/17/release/vc_redist.x64.exe' -ForegroundColor DarkGray
    exit 1
}

Write-Host "PencariMovie Server: $installDir" -ForegroundColor Cyan
$exe = Join-Path $installDir 'pencarimovie-server.exe'
$tmp = Join-Path $env:TEMP ('pm-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
try {
    $url = "$base/$asset"

    # The .sha256 sidecar is served as application/octet-stream, so
    # Invoke-WebRequest hands it back as byte[] and .Trim() throws - which is why
    # an earlier run printed "no sidecar" even though it exists. WebClient
    # decodes it as text.
    #
    # It is fetched FIRST because it also names the release: it hashes the
    # .tar.gz, NOT the exe inside it, so it can never be compared against the
    # installed binary. What it can do is be recorded beside the exe on install -
    # the same name on a later run means this install already IS that release.
    # Without the stamp every re-run re-downloaded 20 MB and then fought the
    # running exe for the file.
    $stamp = Join-Path $installDir '.release-sha256'
    $want = $null
    try {
        $shaText = (New-Object System.Net.WebClient).DownloadString("$url.sha256")
        $want = $shaText.Trim().Split(' ')[0].ToLower()
    } catch { }

    $current = $false
    if ($want -and (Test-Path -LiteralPath $exe) -and (Test-Path -LiteralPath $stamp)) {
        $current = ((Get-Content -LiteralPath $stamp -Raw).Trim() -eq $want)
    }

    if ($current) {
        Write-Host 'Already on the latest build - nothing to download.' -ForegroundColor DarkGray
    } else {
        Write-Host "Downloading $asset"
        Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmp $asset) -UseBasicParsing

        if ($want) {
            $got = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $tmp $asset)).Hash.ToLower()
            if ($want -ne $got) {
                Write-Host 'Checksum mismatch - nothing was installed.' -ForegroundColor Red
                exit 1
            }
            Write-Host 'SHA-256 OK'
        } else {
            Write-Host 'Checksum check skipped: no .sha256 sidecar.' -ForegroundColor DarkGray
        }

        tar.exe -xzf (Join-Path $tmp $asset) -C $tmp
        $rel = Join-Path $tmp 'pencarimovie-server-windows-x86_64'

        # A running server holds pencarimovie-server.exe open, so the copy below
        # failed with "The process cannot access the file ... used by another
        # process". Stop it first and wait for the handle to drop. The tray lives
        # INSIDE the binary, so this removes the icon too.
        $running = Get-Process -Name 'pencarimovie-server' -ErrorAction SilentlyContinue
        if ($running) {
            Write-Host 'Stopping the running server to replace it...' -ForegroundColor DarkGray
            $running | Stop-Process -Force -ErrorAction SilentlyContinue
            $running | Wait-Process -Timeout 15 -ErrorAction SilentlyContinue
            # An instance started by another user (or elevated) survives this
            # silently, and the run would then end with a bare "Already running".
            if (Get-Process -Name 'pencarimovie-server' -ErrorAction SilentlyContinue) {
                Write-Host 'A server started by another user or elevated outlived the stop; close it to finish updating.' -ForegroundColor Yellow
            }
        }

        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        # Copy over the top so storage/ (the bot session) is preserved.
        try {
            Copy-Item -Path (Join-Path $rel '*') -Destination $installDir -Recurse -Force
            if ($want) { Set-Content -LiteralPath $stamp -Value $want -NoNewline -Encoding ASCII }
        } catch {
            Write-Host 'Could not replace the server binary.' -ForegroundColor Red
            Write-Host ('  ' + $_.Exception.Message)
            Write-Host 'The old process survived the stop. Close the tray icon (or this window), or run it from an elevated PowerShell, then re-run:  irm telegra.my/win | iex' -ForegroundColor Yellow
            exit 1
        }
        Get-ChildItem -LiteralPath $installDir -Recurse -File -ErrorAction SilentlyContinue |
            Unblock-File -ErrorAction SilentlyContinue
    }
}
finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

if (-not (Test-Path -LiteralPath $exe)) {
    Write-Host 'The archive did not contain pencarimovie-server.exe.' -ForegroundColor Red
    exit 1
}

# Put the folder on the user PATH so pms resolves from ANY directory. Without
# this the launcher only worked from inside the folder, and a stale copy earlier
# on PATH silently won - which is how an old PHP launcher got invoked.
# (No backticks anywhere in this template literal: they would end it.)
try {
    # PREPEND, not append. Appending left any earlier entry to win, so on a box
    # that already had a checkout or an old install on PATH the word "pms" kept
    # running THAT launcher while the fresh copy sat unused in the folder this
    # script had just written. Dropping the duplicate and going first makes the
    # installed copy the one that runs.
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = @($userPath -split ';' | Where-Object { $_ -ne '' -and $_ -ne $installDir })
    $newPath = (@($installDir) + $parts) -join ';'
    if ($newPath -ne $userPath) {
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Host 'Added to PATH for every new terminal.' -ForegroundColor DarkGray
    }
    # iex runs inside THIS window, so its PATH can be patched here as well. Only
    # the persistent user PATH was being set, which is why the very next line the
    # user typed - "pms uninstall" - came back "not recognized".
    $session = @($env:Path -split ';' | Where-Object { $_ -ne '' -and $_ -ne $installDir })
    $env:Path = (@($installDir) + $session) -join ';'
    Write-Host 'pms works in this window now.' -ForegroundColor DarkGray
} catch {
    Write-Host ('Could not update PATH: ' + $_.Exception.Message) -ForegroundColor DarkGray
}
# Two --tray processes fight over port 8088 and put two icons in the tray, so
# only start when nothing is serving. An update that stopped the old process
# leaves nothing running, so the new build still comes up here.
if (Get-Process -Name 'pencarimovie-server' -ErrorAction SilentlyContinue) {
    Write-Host 'Already running: http://127.0.0.1:8088' -ForegroundColor Green
} else {
    Write-Host 'Starting the server on http://127.0.0.1:8088' -ForegroundColor Green
    # start-tray.bat puts the icon in the notification area and starts the server
    # behind it. Launching the exe directly is what left the previous run with no
    # tray at all. (No backticks in this template literal: they would end it.)
    $tray = Join-Path $installDir 'start-tray.bat'
    if (Test-Path -LiteralPath $tray) {
        Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', 'start-tray.bat' -WorkingDirectory $installDir -WindowStyle Hidden
    } else {
        Start-Process -FilePath $exe -ArgumentList '--open' -WorkingDirectory $installDir -WindowStyle Hidden
    }
    Write-Host 'The tray icon should now be visible; double-click it to open the dashboard.' -ForegroundColor DarkGray
}
`;

/* ── Docker ───────────────────────────────────────────────────────────────── */

const SH_DOCKER = `#!/bin/sh
# PencariMovie Server in a container. Port 8088, state in ./pencarimovie-data.
#
#   curl -fsSL telegra.my/docker | sh
#
# Pulls the PREBUILT multi-arch image (linux/amd64 + linux/arm64) instead of
# building anything. The explicit "docker pull" below is what makes a re-run of
# this command actually pick up a new release: plain "docker run" reuses the
# local image, which is why "docker no rebuild" looked like it kept serving a
# stale server.
set -eu
image="${IMAGE}"
dir="$(pwd)/pencarimovie-data"
mkdir -p "$dir"

# Replace any previous container with the same name.
if docker ps -a --format '{{.Names}}' | grep -qx pencarimovie-server; then
  echo "Removing the existing pencarimovie-server container..."
  docker rm -f pencarimovie-server >/dev/null
fi

echo "Pulling $image ..."
docker pull "$image"

docker run -d --name pencarimovie-server --restart unless-stopped \\
  -p 8088:8088 -v "$dir:/data" "$image"
echo "Started. Open http://127.0.0.1:8088 (state: $dir)"
`;

/* ── landing page ─────────────────────────────────────────────────────────── */

const HTML_HOMEPAGE = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>PencariMovie Server</title>
<meta name="description" content="Self-hosted Telegram media server. One binary per OS, no PHP.">
<link rel="icon" href="/logo.png">
<link rel="apple-touch-icon" href="/logo.png">
<style>
  /* Hallmark · v2.0.0 · theme: Terminal (dealt 1/3) · genre: atmospheric · redesign: full · shape: Session · hero: Committed · nav: a status bar, wordmark and port only · footer: two links, no columns · signature: the install synopsis, one command per row · brand: the mark only, in grayscale · palette: the theme's, taken whole · fonts: system mono, no webfont · scope: page · built: cloudflare-worker-telegra-my.js */
  /* Hallmark · pre-emit critique: P4 H4 E4 S3 R4 V5 · weakest: S, five OS rows could belong to any homelab tool; the hero leads with this machine's own address and every row carries the fact that makes it that OS */
  /* Hallmark · waive 1 · display ui-monospace and body system-ui, two families · this page is served to networks that block third-party hosts, so a font CDN would gate its first paint on a host the reader cannot reach */
  /* Hallmark · dismissed 76 · /logo.png · telegra.my answers 200 for it from this Worker's own route; the render tier reads a file on disk, where no origin routes anything */
  /* Hallmark · dismissed 77 · 3 sections · the page holds exactly what it has: a status bar, the hero, the five-row synopsis and the footer, and no section was invented to reach six */
  :root {
    --color-paper: oklch(12.5% 0.012 150);
    --color-paper-2: oklch(16% 0.016 150);
    --color-paper-3: oklch(20.5% 0.022 150);
    --color-ink: oklch(94% 0.03 140);
    --color-ink-2: oklch(82% 0.05 142);
    --color-muted: oklch(64% 0.06 145);
    --color-rule: oklch(27% 0.028 148);
    --color-rule-2: oklch(38% 0.045 146);
    --color-accent: oklch(87% 0.19 142);
    --color-accent-ink: oklch(15% 0.02 150);
    --color-accent-wash: oklch(24% 0.06 145);
    --color-focus: oklch(90% 0.18 142);

    --font-display: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
    --font-body: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", sans-serif;

    --text-hero: clamp(2.25rem, 5.2vw, 3.75rem);
    --text-section: clamp(2.5rem, 3.2vw, 2.75rem);
    --text-figure: clamp(1.75rem, 3.6vw, 2.75rem);
    --text-index: 1.5rem;
    --text-body: 1rem;
    --text-label: 0.875rem;

    --space-2xs: 0.25rem;
    --space-xs: 0.5rem;
    --space-sm: 0.75rem;
    --space-md: 1rem;
    --space-lg: 1.5rem;
    --space-xl: 2.5rem;
    --space-2xl: 4rem;
    --space-3xl: 6rem;
    --space-4xl: 8rem;

    --rule-hair: 1px;
    --container: 1180px;
    --measure: 62ch;
    --tap: 44px;
    --tap-lg: 3.5rem;
    --dur-micro: 90ms;
    --stretch-display: 112.5%;
    --stretch-body: 87.5%;
  }
  *, *::before, *::after { box-sizing: border-box; }
  html { overflow-x: clip; scrollbar-color: var(--color-rule-2) var(--color-paper); }
  body {
    overflow-x: clip;
    margin: 0;
    background: var(--color-paper);
    color: var(--color-ink);
    font-family: var(--font-body);
    font-size: var(--text-body);
    line-height: 1.65;
    font-variant-numeric: tabular-nums;
    -webkit-font-smoothing: antialiased;
  }
  ::selection { background: var(--color-accent); color: var(--color-accent-ink); }
  h1, h2 {
    margin: 0;
    font-family: var(--font-display);
    font-weight: 500;
    font-stretch: var(--stretch-display);
    letter-spacing: -0.02em;
    line-height: 1.12;
    overflow-wrap: break-word;
    min-width: 0;
    text-wrap: balance;
  }
  h1 { font-size: var(--text-hero); }
  h2 { font-size: var(--text-section); }
  p { margin: 0; }
  code { font-family: var(--font-display); }
  a {
    color: var(--color-ink-2);
    text-decoration: underline dotted var(--color-rule-2);
    text-underline-offset: 0.25em;
  }
  a:hover { color: var(--color-accent); }
  :focus-visible { outline: 2px solid var(--color-focus); outline-offset: 3px; }

  .bar {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    max-width: var(--container);
    margin: 0 auto;
    padding: var(--space-md) var(--space-lg);
    border-bottom: var(--rule-hair) solid var(--color-rule);
    font-size: var(--text-label);
  }
  .bar__mark {
    display: block;
    width: 20px;
    height: 20px;
    filter: grayscale(1) brightness(1.85);
  }
  .bar__port { margin-left: auto; color: var(--color-muted); }

  .hero {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-2xl);
    max-width: var(--container);
    margin: 0 auto;
    padding: var(--space-3xl) var(--space-lg) var(--space-4xl);
  }
  .hero__head { display: grid; gap: var(--space-sm); }
  .hero__lede { max-width: var(--measure); color: var(--color-ink-2); }
  .hero__addr {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xs);
    padding: var(--space-lg) 0;
    border-top: var(--rule-hair) solid var(--color-rule);
    border-bottom: var(--rule-hair) solid var(--color-rule);
  }
  .hero__addr code {
    font-size: var(--text-figure);
    font-stretch: var(--stretch-display);
    font-weight: 400;
    line-height: 1.1;
    color: var(--color-ink);
  }
  .hero__note { color: var(--color-muted); font-size: var(--text-label); }
  .hero__acts { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-md); }
  .hero__index { display: flex; flex-wrap: wrap; }
  .hero__index a {
    padding: var(--space-xs) var(--space-md);
    color: var(--color-ink-2);
    font-family: var(--font-display);
    font-size: var(--text-index);
    text-decoration: none;
  }
  .hero__index a:first-child { padding-left: 0; }
  .hero__index a:hover { color: var(--color-accent); }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: var(--tap);
    padding: 0 var(--space-lg);
    border: var(--rule-hair) solid var(--color-ink-2);
    background: transparent;
    color: var(--color-ink);
    font-family: var(--font-body);
    font-size: var(--text-label);
    text-decoration: none;
    white-space: nowrap;
    cursor: pointer;
    transition: color var(--dur-micro) steps(1), border-color var(--dur-micro) steps(1),
      background-color var(--dur-micro) steps(1);
  }
  .btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
  .btn[aria-disabled="true"],
  .cmd button:disabled {
    opacity: 0.55;
    color: var(--color-muted);
    border-color: var(--color-rule-2);
    cursor: not-allowed;
  }
  .btn--primary { background: var(--color-accent); border-color: var(--color-accent); color: var(--color-accent-ink); }
  .btn--primary:hover,
  .btn--primary:active { background: var(--color-accent-wash); border-color: var(--color-accent); color: var(--color-ink); }

  .install { max-width: var(--container); margin: 0 auto; padding: 0 var(--space-lg) var(--space-4xl); }
  .install h2 { margin-bottom: var(--space-lg); }
  .syn { list-style: none; margin: 0; padding: 0; border-top: var(--rule-hair) solid var(--color-rule); }
  .row {
    display: grid;
    grid-template-columns: minmax(0, 9rem) minmax(0, 1fr) minmax(0, 19rem);
    gap: var(--space-sm) var(--space-lg);
    align-items: center;
    padding: var(--space-md) 0;
    border-bottom: var(--rule-hair) solid var(--color-rule);
  }
  .row__os {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    color: var(--color-ink-2);
    font-size: var(--text-label);
  }
  .row__os svg { flex: none; width: 14px; height: 14px; color: var(--color-accent); }
  .row__note { color: var(--color-muted); font-size: var(--text-label); }

  .cmd {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    min-height: var(--tap-lg);
    padding: var(--space-2xs) var(--space-sm);
    background: var(--color-paper-2);
    border: var(--rule-hair) solid var(--color-rule);
  }
  .cmd__prompt { color: var(--color-muted); font-family: var(--font-display); user-select: none; }
  .cmd code {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    white-space: nowrap;
    color: var(--color-ink);
    font-size: var(--text-label);
  }
  .cmd button {
    min-width: var(--tap);
    min-height: var(--tap);
    padding: 0 var(--space-sm);
    background: transparent;
    border: var(--rule-hair) solid var(--color-rule);
    color: var(--color-ink-2);
    font-family: var(--font-body);
    font-size: var(--text-label);
    cursor: pointer;
    transition: color var(--dur-micro) steps(1), border-color var(--dur-micro) steps(1),
      background-color var(--dur-micro) steps(1);
  }
  .cmd button:hover { color: var(--color-accent); border-color: var(--color-accent); }
  .cmd button:active { background: var(--color-accent); border-color: var(--color-accent); color: var(--color-accent-ink); }
  .cmd--link { background: transparent; border: 0; padding: 0; }

  .foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-lg);
    max-width: var(--container);
    margin: 0 auto;
    padding: var(--space-lg) var(--space-lg) var(--space-3xl);
    border-top: var(--rule-hair) solid var(--color-rule);
    font-size: var(--text-label);
  }

  @media (min-width: 901px) {
    .hero__index {
      border-top: var(--rule-hair) solid var(--color-rule);
      border-bottom: var(--rule-hair) solid var(--color-rule);
    }
    .hero__index a { padding: var(--space-sm) var(--space-md); }
    .hero__index a + a { border-left: var(--rule-hair) solid var(--color-rule); }
  }
  @media (max-width: 900px) {
    .row { grid-template-columns: minmax(0, 1fr); }
    .hero { padding-bottom: var(--space-2xl); }
    .install { padding-bottom: var(--space-3xl); }
  }
  @media (max-width: 600px) {
    .bar { padding: var(--space-sm) var(--space-md); }
    .hero {
      gap: var(--space-lg);
      padding: var(--space-2xl) var(--space-md) var(--space-3xl);
    }
    .hero__addr code { font-size: var(--text-index); }
    .hero__index { gap: var(--space-md); }
    .hero__index a { padding: var(--space-xs) 0; }
    .install { padding-right: var(--space-md); padding-left: var(--space-md); }
    .row { gap: var(--space-md); padding: var(--space-lg) 0; }
    .cmd code {
      font-size: 0.75rem;
      white-space: pre-wrap;
      overflow-wrap: break-word;
    }
    .foot { padding-right: var(--space-md); padding-left: var(--space-md); }
  }
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { transition-duration: 0ms !important; animation-duration: 0ms !important; }
  }
</style>
</head>
<body>
<header class="bar">
  <img class="bar__mark" src="logo.png" alt="PencariMovie" width="22" height="22">
  <span class="bar__name">pencarimovie server</span>
  <span class="bar__port">port 8088</span>
</header>

<main>
  <section class="hero">
    <div class="hero__head">
      <h1>Your media, on your own machine.</h1>
      <p class="hero__lede">One binary, dashboard included, no PHP.</p>
    </div>
    <div class="hero__addr">
      <code>http://127.0.0.1:8088</code>
      <span class="hero__note"># the dashboard, once it runs</span>
    </div>
    <div class="hero__acts">
      <a class="btn btn--primary" href="#install">Install</a>
      <a class="btn" href="/apk">Download the APK</a>
    </div>
    <nav class="hero__index" aria-label="Platforms">
      <a href="#windows">windows</a>
      <a href="#linux">linux</a>
      <a href="#macos">macos</a>
      <a href="#android">android</a>
      <a href="#docker">docker</a>
    </nav>
  </section>

  <section class="install" id="install">
    <h2>Install</h2>
    <ul class="syn">
      <li class="row" id="windows">
        <span class="row__os"><svg aria-hidden="true" viewBox="0 0 24 24" fill="currentColor"><path d="M3 5.5A2.5 2.5 0 0 1 5.5 3h3l1.6 3.2-2 1.2a12 12 0 0 0 5.5 5.5l1.2-2L18 12.5v3a2.5 2.5 0 0 1-2.5 2.5C10 18 6 14 6 8.5A3 3 0 0 1 3 5.5z"/></svg>windows</span>
        <div class="cmd"><span class="cmd__prompt" aria-hidden="true">$</span><code>irm telegra.my/win | iex</code><button type="button" data-copy>Copy</button></div>
        <p class="row__note">Installs to <code>%USERPROFILE%\\pencarimovie-server</code> and starts the tray app.</p>
      </li>
      <li class="row" id="linux">
        <span class="row__os"><svg aria-hidden="true" viewBox="0 0 24 24" fill="currentColor"><path d="M4 6h16v10H4z"/><path d="M2 18h20v2H2z"/></svg>linux</span>
        <div class="cmd"><span class="cmd__prompt" aria-hidden="true">$</span><code>curl -fsSL telegra.my/linux | sh</code><button type="button" data-copy>Copy</button></div>
        <p class="row__note">x86_64 build. Checks the published SHA-256 and keeps <code>storage/</code> on upgrade.</p>
      </li>
      <li class="row" id="macos">
        <span class="row__os"><svg aria-hidden="true" viewBox="0 0 24 24" fill="currentColor"><path d="M16.5 3a4 4 0 0 1 3.9 3.2A3.6 3.6 0 0 1 21 9.4c0 1.3-.7 2.4-1.7 3.1V13a4 4 0 0 1-4 4h-1v4h-2v-4H9.7a4 4 0 0 1-7.4-2.4 3.7 3.7 0 0 1-.1-5A4.1 4.1 0 0 1 6 4.3 4 4 0 0 1 9.6 3z"/></svg>macos</span>
        <div class="cmd"><span class="cmd__prompt" aria-hidden="true">$</span><code>curl -fsSL telegra.my/mac | sh</code><button type="button" data-copy>Copy</button></div>
        <p class="row__note">Apple Silicon and Intel builds, picked from <code>uname -m</code> by the same installer as Linux.</p>
      </li>
      <li class="row" id="android">
        <span class="row__os"><svg aria-hidden="true" viewBox="0 0 24 24" fill="currentColor"><path d="M7 2h10a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm3 17h4v1.5h-4z"/></svg>android</span>
        <div class="cmd cmd--link"><a class="btn btn--primary" href="/apk">Download the APK</a></div>
        <p class="row__note">Standalone app with the server engine built in.</p>
      </li>
      <li class="row" id="docker">
        <span class="row__os"><svg aria-hidden="true" viewBox="0 0 24 24" fill="currentColor"><path d="M3 7h18v3H3zm0 5h18v3H3zm0 5h18v3H3z"/></svg>docker</span>
        <div class="cmd"><span class="cmd__prompt" aria-hidden="true">$</span><code>curl -fsSL telegra.my/docker | sh</code><button type="button" data-copy>Copy</button></div>
        <p class="row__note">Container on port 8088, state in <code>./pencarimovie-data</code>.</p>
      </li>
    </ul>
  </section>

  <footer class="foot">
    <a href="https://github.com/${GITHUB_TARGET}">Source</a>
    <a href="https://github.com/${GITHUB_TARGET}/releases/latest">Releases</a>
  </footer>
</main>
<script>
  document.querySelectorAll('[data-copy]').forEach(function (btn) {
    btn.addEventListener('click', function () {
      var text = btn.parentElement.querySelector('code').textContent;
      var done = function () {
        var old = btn.textContent;
        btn.textContent = 'Copied';
        setTimeout(function () { btn.textContent = old; }, 1200);
      };
      if (navigator.clipboard) {
        navigator.clipboard.writeText(text).then(done, function () {});
        return;
      }
      var ta = document.createElement('textarea');
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      try { document.execCommand('copy'); done(); } catch (e) {}
      document.body.removeChild(ta);
    });
  });
</script>
</body>
</html>
`;

function text(body, cache) {
  return new Response(body, {
    status: 200,
    headers: {
      "Content-Type": "text/plain; charset=utf-8",
      "Access-Control-Allow-Origin": "*",
      "Cache-Control": cache || "no-store, no-cache, must-revalidate",
    },
  });
}

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname.toLowerCase().replace(/\/+$/, "");

    // 1. DPI-firewall fallback: reverse-proxy the origin's API over Cloudflare.
    //    Checked FIRST so /wp-json is never shadowed by a shortcut below.
    if (path.startsWith("/wp-json") || path.startsWith("/wp-admin")) {
      const targetUrl = new URL(request.url);
      targetUrl.hostname = "pencarimovie.com";
      targetUrl.protocol = "https:";
      targetUrl.port = "";
      const proxyReq = new Request(targetUrl.toString(), request);
      proxyReq.headers.set("Host", "pencarimovie.com");
      return fetch(proxyReq);
    }

    // 2. Landing page.
    if (path === "" || path === "/") {
      return new Response(HTML_HOMEPAGE, {
        status: 200,
        headers: {
          "Content-Type": "text/html; charset=utf-8",
          "Access-Control-Allow-Origin": "*",
          "Cache-Control": "public, max-age=300",
        },
      });
    }

    // 3. Logo, proxied over Cloudflare so a network that blocks the origin (or
    //    GitHub) still renders it. The origin has no /logo.png at all.
    if (path === "/logo.png" || path === "/favicon.ico") {
      return fetch(`${RAW_BASE}/public/logo.png`, { cf: { cacheTtl: 86400 } });
    }

    // 4. Installers, served inline so the one-liners work unchanged.
    if (path === "/win" || path === "/win.ps1" || path === "/windows") {
      return text(PS1_INSTALL);
    }
    if (path === "/linux" || path === "/linux.sh" || path === "/sh") {
      return text(SH_INSTALL);
    }
    if (path === "/mac" || path === "/macos" || path === "/darwin" || path === "/osx" || path === "/mac.sh") {
      return text(SH_INSTALL);
    }
    if (path === "/docker" || path === "/docker.sh") {
      return text(SH_DOCKER);
    }

    // 5. Redirects.
    // /android, /apk, /app, and legacy /termux redirect directly to the APK download.
    if (
      path === "/apk" ||
      path === "/app" ||
      path === "/android" ||
      path === "/android.sh" ||
      path === "/termux" ||
      path === "/termux.sh"
    ) {
      return Response.redirect(APK_URL, 302);
    }
    if (path === "/github" || path === "/repo") {
      return Response.redirect(`https://github.com/${GITHUB_TARGET}`, 302);
    }
    if (path === "/releases") {
      return Response.redirect(RELEASE_PAGE, 302);
    }

    return text(
      "Not found. Available: /win, /linux, /mac, /android, /docker, /apk, /github\n",
      "no-store"
    );
  },
};
