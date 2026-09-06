/**
 * Cloudflare Worker for telegra.my
 *
 * Provides ultra-short installation and download shortcuts for PencariMovie:
 *
 * Installation Shortcuts:
 *   - telegra.my/win    -> Windows installer script (.bat)
 *   - telegra.my/linux  -> Linux installer script (.sh)
 *   - telegra.my/termux -> Termux Android installer script (.sh)
 *   - telegra.my/apk    -> Standalone Android APK download
 *   - telegra.my/github -> GitHub repository
 *
 * Usage examples in terminal:
 *   Windows (PowerShell):
 *     irm telegra.my/win | iex
 *
 *   Linux:
 *     curl -fsSL telegra.my/linux | bash
 *
 *   Termux:
 *     curl -fsSL telegra.my/termux | bash
 */

// Inline script contents to serve directly with 200 OK (no redirect hops or GitHub raw dependencies)

const PS1_SCRIPT = `# PencariMovie Server - Windows One-Line Installer & Launcher
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $HOME }
$installDir = Join-Path $homeDir "pencarimovie-server"

if (-not (Test-Path -LiteralPath $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$batFile = Join-Path $installDir "pencarimovie-windows.bat"
$url = "https://raw.githubusercontent.com/aiskendi/pencarimovie-downloader/main/pencarimovie-windows.bat"

Write-Host "PencariMovie Server: $installDir" -ForegroundColor Cyan
try {
    Invoke-RestMethod -Uri $url -OutFile $batFile
} catch {
    $fallbackUrl = "https://raw.githubusercontent.com/aiskendi/pencarimovie-server/main/pencarimovie-windows.bat"
    Invoke-RestMethod -Uri $fallbackUrl -OutFile $batFile
}

Push-Location $installDir
try {
    & $batFile @args
} finally {
    Pop-Location
}
`;

const GITHUB_TARGET = "aiskendi/pencarimovie-downloader";
const GITHUB_FALLBACK = "aiskendi/pencarimovie-server";

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname.toLowerCase().replace(/\/+$/, "");

    // Direct inline serving for Windows PowerShell one-liner
    if (path === "/win" || path === "/win.ps1" || path === "/windows") {
      return new Response(PS1_SCRIPT, {
        status: 200,
        headers: {
          "Content-Type": "text/plain; charset=utf-8",
          "Access-Control-Allow-Origin": "*",
          "Cache-Control": "public, max-age=60",
        },
      });
    }

    // Windows .bat redirect
    if (path === "/win.bat") {
      return Response.redirect(`https://raw.githubusercontent.com/${GITHUB_TARGET}/main/pencarimovie-windows.bat`, 302);
    }

    // Linux script redirect
    if (path === "/linux" || path === "/linux.sh" || path === "/sh") {
      return Response.redirect(`https://raw.githubusercontent.com/${GITHUB_TARGET}/main/pencarimovie-linux.sh`, 302);
    }

    // Termux script redirect
    if (path === "/termux" || path === "/termux.sh") {
      return Response.redirect(`https://raw.githubusercontent.com/${GITHUB_TARGET}/main/pencarimovie-termux.sh`, 302);
    }

    // Android APK download
    if (path === "/apk" || path === "/app") {
      return Response.redirect(`https://github.com/${GITHUB_FALLBACK}/releases/latest/download/pencarimovie_arm64-v8a.apk`, 302);
    }

    // GitHub Repo
    if (path === "" || path === "/" || path === "/github" || path === "/repo") {
      return Response.redirect(`https://github.com/${GITHUB_TARGET}`, 302);
    }

    return new Response("Not found. Available shortcuts: /win, /linux, /termux, /apk, /github\n", {
      status: 404,
      headers: { "Content-Type": "text/plain; charset=utf-8" },
    });
  },
};
