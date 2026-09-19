param(
    [string]$Root = "",
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($Root)) {
    $Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$targets = @(
    'frankenphp-windows-x86_64.zip',
    'frankenphp-linux-x86_64',
    'frankenphp-linux-x86_64-gnu',
    'frankenphp-linux-x86_64-mimalloc',
    'frankenphp-linux-aarch64',
    'frankenphp-linux-aarch64-gnu',
    'frankenphp-mac-arm64',
    'frankenphp-mac-x86_64'
)

$tagFile = Join-Path $Root '.frankenphp-tag'
$installedTag = if (Test-Path $tagFile) { (Get-Content -Raw $tagFile).Trim() } else { '' }

$headers = @{ 'User-Agent' = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) pencarimovie-release-builder' }
$latestTag = $null
$assetUrls = @{}

try {
    $apiRel = Invoke-RestMethod -Uri 'https://api.github.com/repos/php/frankenphp/releases/latest' -Headers $headers -TimeoutSec 15
    $latestTag = $apiRel.tag_name
    foreach ($a in $apiRel.assets) {
        $assetUrls[$a.name] = $a.browser_download_url
    }
    Write-Host ('  Latest FrankenPHP release tag on GitHub: ' + $latestTag) -ForegroundColor Cyan
}
catch {
    Write-Warning ('  GitHub API query failed: ' + $_.Exception.Message + '. Falling back to direct /releases/latest/download/ URLs.')
}

$needDownload = @()
foreach ($t in $targets) {
    $p = Join-Path $Root $t
    $missingOrCorrupt = (-not (Test-Path $p)) -or ((Get-Item $p).Length -lt 1000)
    $versionOutdated = ($Force.IsPresent) -or ($latestTag -and $installedTag -ne '' -and $installedTag -ne $latestTag)
    
    if ($missingOrCorrupt -or $versionOutdated) {
        $needDownload += $t
    }
}

if ($needDownload.Count -eq 0) {
    Write-Host '  All latest FrankenPHP binaries are present and up to date.' -ForegroundColor Green
    exit 0
}

Write-Host ('  Found ' + $needDownload.Count + ' FrankenPHP asset(s) to fetch...') -ForegroundColor Yellow

$wc = New-Object System.Net.WebClient
$wc.Headers.Add('User-Agent', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) pencarimovie-release-builder')

try {
    foreach ($t in $needDownload) {
        $dest = Join-Path $Root $t
        $url = if ($assetUrls.ContainsKey($t)) { $assetUrls[$t] } else { 'https://github.com/php/frankenphp/releases/latest/download/' + $t }
        Write-Host ('  Downloading ' + $t + ' ... ') -NoNewline
        $tmpFile = $dest + '.tmp'
        if (Test-Path $tmpFile) {
            Remove-Item -Force $tmpFile
        }
        try {
            $wc.DownloadFile($url, $tmpFile)
            if ((Get-Item $tmpFile).Length -lt 1000) {
                throw 'Downloaded file is unexpectedly small.'
            }
            if (Test-Path $dest) {
                Remove-Item -Force $dest
            }
            Move-Item -Force $tmpFile $dest
            $szMb = [math]::Round(((Get-Item $dest).Length / 1MB), 2)
            Write-Host ('OK (' + $szMb + ' MB)') -ForegroundColor Green
        }
        catch {
            if (Test-Path $tmpFile) {
                Remove-Item -Force $tmpFile
            }
            Write-Host 'FAILED' -ForegroundColor Red
            Write-Error ('Failed to download ' + $t + ' from ' + $url + ': ' + $_.Exception.Message)
            exit 1
        }
    }
    
    if ($latestTag) {
        Set-Content -Path $tagFile -Value $latestTag -NoNewline
    }
}
finally {
    $wc.Dispose()
}
