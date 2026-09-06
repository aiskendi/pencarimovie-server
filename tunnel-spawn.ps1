param(
    [Parameter(Mandatory = $true)]
    [string]$Bin,
    [Parameter(Mandatory = $true)]
    [string]$LocalUrl,
    [Parameter(Mandatory = $true)]
    [string]$Config,
    [Parameter(Mandatory = $true)]
    [string]$LogFile,
    [Parameter(Mandatory = $true)]
    [string]$PidFile,
    [Parameter(Mandatory = $false)]
    [string]$Metrics = '127.0.0.1:20241'
)

# Detached hidden cloudflared start for Windows.
# Use --logfile (cloudflared writes after handshake) instead of Start-Process
# stdout redirect, which can leave 0-byte logs. --metrics exposes /quicktunnel
# so PHP can read the live hostname instead of a stale log scrape.

$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $Bin)) {
    throw "cloudflared binary not found: $Bin"
}

$logDir = Split-Path -Parent $LogFile
if ($logDir -and -not (Test-Path -LiteralPath $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}

$pidDir = Split-Path -Parent $PidFile
if ($pidDir -and -not (Test-Path -LiteralPath $pidDir)) {
    New-Item -ItemType Directory -Path $pidDir -Force | Out-Null
}

$errLog = [System.IO.Path]::ChangeExtension($LogFile, '.err.log')
foreach ($path in @($LogFile, $errLog)) {
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
    }
}

$env:TUNNEL_TRANSPORT_PROTOCOL = 'http2'

$argList = @(
    'tunnel',
    '--url', $LocalUrl,
    '--config', $Config,
    '--logfile', $LogFile,
    '--metrics', $Metrics,
    '--no-autoupdate',
    '--retries', '99'
)

# Use WScript.Shell / ProcessStartInfo so stdout/stderr redirection does not hang Start-Process
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $Bin
$psi.Arguments = ($argList | ForEach-Object {
        if ($_ -match '[\s"]') {
            '"' + ($_ -replace '"', '\"') + '"'
        }
        else {
            $_
        }
    }) -join ' '
$psi.WorkingDirectory = $PSScriptRoot
$psi.CreateNoWindow = $true
$psi.UseShellExecute = $false
$psi.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden

$proc = [System.Diagnostics.Process]::Start($psi)
if ($null -eq $proc) {
    throw "Failed to create cloudflared process"
}

Set-Content -LiteralPath $PidFile -Value $proc.Id -Encoding ASCII
Write-Output $proc.Id
