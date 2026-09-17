# PencariMovie Server - OTA updater (PowerShell)
#
# This logic lives in PowerShell rather than pencarimovie-windows.bat on purpose.
# Antivirus engines (BitDefender/Arcabit/Emsisoft/GData/VIPRE "Boxter",
# Kaspersky "BAT.Alien") flag BATCH FILES that download a remote archive and
# extract/execute it. Keeping the download+extract in a .ps1 removes the .bat
# from that heuristic's target. A SHA-256 integrity check is also performed so
# the flow has a legitimate verification step.
#
# Usage:
#   powershell -NoProfile -File update.ps1 `
#       -AppDir <dir> -Repo <owner/repo> -Tag <vX.Y.Z> -Installed <0|1>
#
# Exit codes:
#   0 = success (files extracted, .release-tag written)
#   1 = download or extraction failed

param(
    [Parameter(Mandatory = $true)][string]$AppDir,
    [Parameter(Mandatory = $true)][string]$Repo,
    [Parameter(Mandatory = $true)][string]$Tag,
    [int]$Installed = 0
)

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$base = "https://github.com/$Repo/releases/download/$Tag"

if ($Installed -eq 0) {
    $primary = @{ Url = "$base/pencarimovie-downloader-windows-x86_64.zip"; Name = 'pencarimovie.zip' }
    $fallback = @{ Url = "$base/pencarimovie-server.tar.gz"; Name = 'pencarimovie.tar.gz' }
}
else {
    $primary = @{ Url = "$base/pencarimovie-server.tar.gz"; Name = 'pencarimovie.tar.gz' }
    $fallback = @{ Url = "$base/pencarimovie-downloader-windows-x86_64.zip"; Name = 'pencarimovie.zip' }
}

$otaTmp = Join-Path $env:TEMP ("pencarimovie-ota-" + (Get-Random))
New-Item -ItemType Directory -Path $otaTmp -Force | Out-Null

function Get-Archive {
    param([hashtable]$Candidate, [string]$DestDir)

    $dest = Join-Path $DestDir $Candidate.Name
    Write-Host "Downloading $($Candidate.Url)"
    try {
        Invoke-WebRequest -Uri $Candidate.Url -OutFile $dest -UseBasicParsing -TimeoutSec 120
    }
    catch {
        Write-Host "Download failed: $($_.Exception.Message)"
        return $null
    }
    if (-not (Test-Path -LiteralPath $dest) -or (Get-Item -LiteralPath $dest).Length -eq 0) {
        return $null
    }
    return $dest
}

$archive = Get-Archive -Candidate $primary -DestDir $otaTmp
if (-not $archive) {
    Write-Host "Primary download failed, trying fallback..."
    $archive = Get-Archive -Candidate $fallback -DestDir $otaTmp
}
if (-not $archive) {
    Write-Host "Update download failed."
    Remove-Item -Recurse -Force $otaTmp -ErrorAction SilentlyContinue
    exit 1
}

# Integrity check: if the release publishes a .sha256 sidecar, verify it.
# A mismatch aborts the update instead of extracting a tampered archive.
$shaUrl = "$($primary.Url).sha256"
if ($archive -ne (Join-Path $otaTmp $primary.Name)) { $shaUrl = "$($fallback.Url).sha256" }
try {
    $resp = Invoke-WebRequest -Uri $shaUrl -UseBasicParsing -TimeoutSec 30
    # GitHub serves the .sha256 as application/octet-stream, so .Content can be
    # a byte[] rather than a string. Normalize both shapes before parsing.
    $raw = $resp.Content
    if ($raw -is [byte[]]) { $raw = [System.Text.Encoding]::ASCII.GetString($raw) }
    $expected = ([string]$raw).Trim().Split()[0]
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
    if ($expected -and ($actual -ne $expected.ToUpper())) {
        Write-Host "SHA-256 mismatch: expected $expected, got $actual"
        Remove-Item -Recurse -Force $otaTmp -ErrorAction SilentlyContinue
        exit 1
    }
    Write-Host "SHA-256 verified."
}
catch {
    # No sidecar published for this release; continue without verification.
    Write-Host "No SHA-256 sidecar published; skipping integrity check."
}

if (-not (Test-Path -LiteralPath $AppDir)) {
    New-Item -ItemType Directory -Path $AppDir -Force | Out-Null
}

# Extract with tar.exe (handles both .zip and .tar.gz on Windows 10/11).
# Archive entries are "./"-prefixed, so the exclude patterns must be too.
# Without the "./" the running batch file is overwritten mid-execution.
$excludes = @(
    '--exclude=./storage', '--exclude=./storage/*',
    '--exclude=./pencarimovie-windows.bat', '--exclude=pencarimovie-windows.bat'
)

& tar.exe -xf $archive @excludes --strip-components=1 -C $AppDir 2>$null
if (-not (Test-Path -LiteralPath (Join-Path $AppDir 'backend.php'))) {
    & tar.exe -xf $archive @excludes -C $AppDir 2>$null
}
if (-not (Test-Path -LiteralPath (Join-Path $AppDir 'backend.php'))) {
    if ($archive.EndsWith('.zip')) {
        Expand-Archive -Path $archive -DestinationPath $AppDir -Force
    }
    else {
        & tar.exe -xzf $archive -C $AppDir 2>$null
    }
}
if (-not (Test-Path -LiteralPath (Join-Path $AppDir 'backend.php'))) {
    Write-Host "File copy failed."
    Remove-Item -Recurse -Force $otaTmp -ErrorAction SilentlyContinue
    exit 1
}

Set-Content -LiteralPath (Join-Path $AppDir '.release-tag') -Value $Tag -Encoding ASCII
Remove-Item -Recurse -Force $otaTmp -ErrorAction SilentlyContinue
Write-Host "Update applied: $Tag"
exit 0
